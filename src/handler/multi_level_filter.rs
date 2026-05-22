use std::collections::HashSet;
use std::sync::Arc;

use axum::{extract::State, Json};
use serde_json::Value;
use tokio::sync::Semaphore;

use crate::api_models::multi_level_filter::{
    MonthlyMaCrossItem, MonthlyMaCrossRequest, MonthlyMaCrossResponse, PlateBrief, SkippedStock,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_snapshot::{self, LatestSnapshotFields};
use crate::services::kline_service;
use crate::services::monthly_ma_cross::{self, EvalOutcome};
use crate::services::monthly_ma_cross_screen_cache::{
    ma_cross_screen_fingerprint, shanghai_calendar_date_now,
};

/// 月线并发（经 [`proxy_get_json`] 共享代理出站）。默认 32，`MULTI_LEVEL_FILTER_KLINE_CONCURRENCY` 可改，上限 128。
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

enum TaskOutcome {
    Hit(MonthlyMaCrossItem),
    Skipped(SkippedStock),
}

async fn process_stock(
    semaphore: Arc<Semaphore>,
    row: LatestSnapshotFields,
    anchor_year: Option<i32>,
    anchor_month: Option<u32>,
) -> TaskOutcome {
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
            return TaskOutcome::Skipped(SkippedStock {
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
                return TaskOutcome::Skipped(SkippedStock {
                    stock_code,
                    stock_name,
                    plates: plate_list,
                    reason: format!("monthly kline fetch: {e}"),
                });
            }
            Ok(r) => r.parsed,
        };

    match monthly_ma_cross::eval_monthly_ma5_cross_ma20(klines, anchor_year, anchor_month) {
        Err(e) => TaskOutcome::Skipped(SkippedStock {
            stock_code,
            stock_name,
            plates: plate_list,
            reason: e,
        }),
        Ok(EvalOutcome::Miss { reason }) => TaskOutcome::Skipped(SkippedStock {
            stock_code,
            stock_name,
            plates: plate_list,
            reason,
        }),
        Ok(EvalOutcome::Hit(m)) => TaskOutcome::Hit(MonthlyMaCrossItem {
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

pub async fn monthly_ma_cross_screen(
    State(state): State<AppState>,
    Json(req): Json<MonthlyMaCrossRequest>,
) -> Result<Json<MonthlyMaCrossResponse>, AppError> {
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
    let fingerprint = ma_cross_screen_fingerprint(&req, &snapshots);
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
        return Ok(Json(hit));
    }

    let parallel = ma_cross_kline_concurrency();
    tracing::info!(
        target: "multi_level_filter",
        stock_count,
        kline_parallel = parallel,
        "monthly-ma-cross computing (eastmoney proxy, high concurrency semaphore)"
    );

    let sem = Arc::new(Semaphore::new(parallel));
    let ay = req.anchor_year;
    let am = req.anchor_month;

    let mut join_set = tokio::task::JoinSet::new();
    for row in snapshots {
        let sem = Arc::clone(&sem);
        join_set.spawn(async move { process_stock(sem, row, ay, am).await });
    }

    let mut items = Vec::new();
    let mut skipped = Vec::new();

    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(TaskOutcome::Hit(item)) => items.push(item),
            Ok(TaskOutcome::Skipped(s)) => skipped.push(s),
            Err(err) => {
                tracing::warn!("multi_level_filter join error: {:?}", err);
            }
        }
    }

    items.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));
    skipped.sort_by(|a, b| a.stock_code.cmp(&b.stock_code));

    let response = MonthlyMaCrossResponse { items, skipped };
    state
        .ma_cross_screen_cache
        .insert(today_sh, fingerprint, response.clone())
        .await;

    Ok(Json(response))
}
