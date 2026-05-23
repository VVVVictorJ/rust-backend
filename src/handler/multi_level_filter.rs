use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::Value;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api_models::multi_level_filter::{
    DailyAfterMonthlyMaCrossResponse, MonthlyMaCrossItem, MonthlyMaCrossRequest,
    MonthlyMaCrossResponse, PlateBrief, SkippedStock,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_snapshot::{self, LatestSnapshotFields};
use crate::services::daily_ma_cross;
use crate::services::kline_service;
use crate::services::monthly_ma_cross::{self, EvalOutcome};
use crate::services::monthly_ma_cross_screen_cache::{
    ma_cross_screen_fingerprint, shanghai_calendar_date_now,
};

/// 月线 / 日线二次筛查共用：`MULTI_LEVEL_FILTER_KLINE_CONCURRENCY`，默认 32，上限 128。
fn ma_cross_kline_concurrency() -> usize {
    std::env::var("MULTI_LEVEL_FILTER_KLINE_CONCURRENCY")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(32)
        .clamp(1, 128)
}

fn plates_from_db(v: &Value) -> Vec<PlateBrief> {
    serde_json::from_value::<Vec<PlateBrief>>(v.clone()).unwrap_or_default()
}

fn row_matches_any_plate(row: &LatestSnapshotFields, filter: &HashSet<String>) -> bool {
    if filter.is_empty() {
        return true;
    }
    let arr = plates_from_db(&row.plates);
    arr.iter().any(|p| filter.contains(&p.plate_code))
}

enum MonthlyTaskOutcome {
    Hit(MonthlyMaCrossItem),
    Skipped(SkippedStock),
}

async fn process_stock_monthly(
    semaphore: Arc<Semaphore>,
    row: LatestSnapshotFields,
    anchor_year: Option<i32>,
    anchor_month: Option<u32>,
) -> MonthlyTaskOutcome {
    let LatestSnapshotFields {
        stock_code,
        stock_name,
        latest_price,
        plates,
    } = row;

    let plate_list = plates_from_db(&plates);

    let permit = match semaphore.acquire_owned().await {
        Ok(p) => p,
        Err(_) => {
            return MonthlyTaskOutcome::Skipped(SkippedStock {
                stock_code,
                stock_name,
                plates: plate_list,
                reason: "concurrency semaphore closed".to_string(),
            });
        }
    };

    let _permit_guard = permit;

    let klines =
        match kline_service::fetch_and_parse_monthly_kline_via_proxy_only(&stock_code).await {
            Err(e) => {
                return MonthlyTaskOutcome::Skipped(SkippedStock {
                    stock_code,
                    stock_name,
                    plates: plate_list,
                    reason: format!("monthly kline fetch: {e}"),
                });
            }
            Ok(r) => r.parsed,
        };

    match monthly_ma_cross::eval_monthly_ma5_cross_ma20(klines, anchor_year, anchor_month) {
        Err(e) => MonthlyTaskOutcome::Skipped(SkippedStock {
            stock_code,
            stock_name,
            plates: plate_list,
            reason: e,
        }),
        Ok(EvalOutcome::Miss { reason }) => MonthlyTaskOutcome::Skipped(SkippedStock {
            stock_code,
            stock_name,
            plates: plate_list,
            reason,
        }),
        Ok(EvalOutcome::Hit(m)) => MonthlyTaskOutcome::Hit(MonthlyMaCrossItem {
            stock_code,
            stock_name,
            latest_price,
            plates: plate_list,
            ma5_current: Some(m.ma5_current),
            ma20_current: Some(m.ma20_current),
            ma5_prev: Some(m.ma5_prev),
            ma20_prev: Some(m.ma20_prev),
        }),
    }
}

async fn compute_monthly_scan_uncached(
    snapshots: Vec<LatestSnapshotFields>,
    anchor_year: Option<i32>,
    anchor_month: Option<u32>,
) -> MonthlyMaCrossResponse {
    let parallel = ma_cross_kline_concurrency();
    let sem = Arc::new(Semaphore::new(parallel));

    let mut join_set = JoinSet::new();
    for row in snapshots {
        let sem_c = Arc::clone(&sem);
        join_set.spawn(async move {
            process_stock_monthly(sem_c, row, anchor_year, anchor_month).await
        });
    }

    let mut items = Vec::new();
    let mut skipped = Vec::new();

    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(MonthlyTaskOutcome::Hit(item)) => items.push(item),
            Ok(MonthlyTaskOutcome::Skipped(s)) => skipped.push(s),
            Err(err) => {
                tracing::warn!("multi_level_filter monthly join error: {:?}", err);
            }
        }
    }

    items.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));
    skipped.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));

    MonthlyMaCrossResponse { items, skipped }
}

/// 与同指纹月线缓存对齐：缓存命中则不重复批量拉月线。
async fn resolve_monthly_ma_cross_response(
    state: &AppState,
    req: &MonthlyMaCrossRequest,
) -> Result<MonthlyMaCrossResponse, AppError> {
    if req.anchor_year.is_some() ^ req.anchor_month.is_some() {
        return Err(AppError::BadRequest(
            "anchor_year and anchor_month must both be set or both omitted".to_string(),
        ));
    }

    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let snapshots_all = stock_snapshot::list_latest_snapshot_fields_per_stock(&mut conn)
        .map_err(|_| AppError::InternalServerError)?;

    let filter_set: HashSet<String> = req.filter_plate_codes.iter().cloned().collect();
    let snapshots: Vec<LatestSnapshotFields> = if filter_set.is_empty() {
        snapshots_all
    } else {
        snapshots_all
            .into_iter()
            .filter(|r| row_matches_any_plate(r, &filter_set))
            .collect()
    };

    let today_sh = shanghai_calendar_date_now();
    let fingerprint = ma_cross_screen_fingerprint(req, &snapshots);
    let stock_count = snapshots.len();

    if let Some(hit) = state
        .ma_cross_screen_cache
        .try_hit(today_sh, &fingerprint)
        .await
    {
        tracing::debug!(
            target: "multi_level_filter",
            stock_count,
            "ma_cross_screen_cache hit"
        );
        return Ok(hit);
    }

    tracing::info!(
        target: "multi_level_filter",
        stock_count,
        kline_parallel = ma_cross_kline_concurrency(),
        "monthly-ma-cross computing (eastmoney proxy, high concurrency semaphore)"
    );

    let ay = req.anchor_year;
    let am = req.anchor_month;
    let response = compute_monthly_scan_uncached(snapshots, ay, am).await;
    state
        .ma_cross_screen_cache
        .insert(today_sh, fingerprint, response.clone())
        .await;

    Ok(response)
}

enum DailyTaskOutcome {
    Hit(MonthlyMaCrossItem),
    Skipped(SkippedStock),
}

async fn process_daily_refine(
    semaphore: Arc<Semaphore>,
    monthly_hit: MonthlyMaCrossItem,
) -> DailyTaskOutcome {
    let permit = match semaphore.acquire_owned().await {
        Ok(p) => p,
        Err(_) => {
            return DailyTaskOutcome::Skipped(SkippedStock {
                stock_code: monthly_hit.stock_code.clone(),
                stock_name: monthly_hit.stock_name.clone(),
                plates: monthly_hit.plates.clone(),
                reason: "daily refine: concurrency semaphore closed".to_string(),
            });
        }
    };

    let _permit_guard = permit;

    let klines =
        match kline_service::fetch_and_parse_daily_kline_via_proxy_only(&monthly_hit.stock_code)
            .await
        {
            Err(e) => {
                return DailyTaskOutcome::Skipped(SkippedStock {
                    stock_code: monthly_hit.stock_code.clone(),
                    stock_name: monthly_hit.stock_name.clone(),
                    plates: monthly_hit.plates.clone(),
                    reason: format!("daily kline fetch: {e}"),
                });
            }
            Ok(r) => r.parsed,
        };

    match daily_ma_cross::eval_daily_ma5_cross_ma20(klines) {
        Err(e) => DailyTaskOutcome::Skipped(SkippedStock {
            stock_code: monthly_hit.stock_code.clone(),
            stock_name: monthly_hit.stock_name.clone(),
            plates: monthly_hit.plates.clone(),
            reason: e,
        }),
        Ok(EvalOutcome::Miss { reason }) => DailyTaskOutcome::Skipped(SkippedStock {
            stock_code: monthly_hit.stock_code.clone(),
            stock_name: monthly_hit.stock_name.clone(),
            plates: monthly_hit.plates.clone(),
            reason,
        }),
        Ok(EvalOutcome::Hit(m)) => DailyTaskOutcome::Hit(MonthlyMaCrossItem {
            stock_code: monthly_hit.stock_code,
            stock_name: monthly_hit.stock_name,
            latest_price: monthly_hit.latest_price,
            plates: monthly_hit.plates,
            ma5_current: Some(m.ma5_current),
            ma20_current: Some(m.ma20_current),
            ma5_prev: Some(m.ma5_prev),
            ma20_prev: Some(m.ma20_prev),
        }),
    }
}

async fn compute_daily_refine(monthly_hits: Vec<MonthlyMaCrossItem>) -> MonthlyMaCrossResponse {
    if monthly_hits.is_empty() {
        return MonthlyMaCrossResponse {
            items: Vec::new(),
            skipped: Vec::new(),
        };
    }

    let parallel = ma_cross_kline_concurrency();
    let sem = Arc::new(Semaphore::new(parallel));
    tracing::info!(
        target: "multi_level_filter",
        monthly_hit_count = monthly_hits.len(),
        kline_parallel = parallel,
        "daily-ma-cross-after-monthly: refining monthly hits with daily bars"
    );

    let mut join_set = JoinSet::new();
    for hit in monthly_hits {
        let sem_c = Arc::clone(&sem);
        join_set.spawn(async move { process_daily_refine(sem_c, hit).await });
    }

    let mut items = Vec::new();
    let mut skipped = Vec::new();

    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(DailyTaskOutcome::Hit(item)) => items.push(item),
            Ok(DailyTaskOutcome::Skipped(s)) => skipped.push(s),
            Err(err) => {
                tracing::warn!("multi_level_filter daily refine join error: {:?}", err);
            }
        }
    }

    items.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));
    skipped.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));

    MonthlyMaCrossResponse { items, skipped }
}

pub async fn monthly_ma_cross_screen(
    State(state): State<AppState>,
    Json(req): Json<MonthlyMaCrossRequest>,
) -> Result<Json<MonthlyMaCrossResponse>, AppError> {
    let response = resolve_monthly_ma_cross_response(&state, &req).await?;
    Ok(Json(response))
}

/// 先解析月线扫描（与同参缓存对齐），仅对月线**命中**标的拉日线判断是否 MA5×MA20 刚上穿；不写库。
/// 一并返回月线结果供前端在未先点月线按钮时仍可同步表格。
pub async fn daily_ma_cross_after_monthly_screen(
    State(state): State<AppState>,
    Json(req): Json<MonthlyMaCrossRequest>,
) -> Result<Json<DailyAfterMonthlyMaCrossResponse>, AppError> {
    let monthly = resolve_monthly_ma_cross_response(&state, &req).await?;
    let items_clone = monthly.items.clone();
    let refined = compute_daily_refine(items_clone).await;
    Ok(Json(DailyAfterMonthlyMaCrossResponse {
        monthly,
        daily_refinement: refined,
    }))
}
