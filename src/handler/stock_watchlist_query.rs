use axum::{
    extract::State,
    Json,
};
use chrono::{Utc, NaiveDate};
use serde_json;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::api_models::stock_trade_date_query::PlateInfo;
use crate::api_models::stock_watchlist_query::{
    WatchlistQueryRequest, WatchlistQueryItem, WatchlistQueryResponse,
    WatchlistDetailRequest, WatchlistDetailItem, WatchlistDetailResponse,
    WatchlistKlineRequest, WatchlistKlineItem, WatchlistKlineResponse,
    WatchlistFillKlineRequest, WatchlistFillKlineResponse, StockFillKlineDetail,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::{stock_watchlist_query, stock_watchlist, daily_kline};
use crate::services::kline_service;
use crate::utils::http_client;

/// 查询观察表股票列表
pub async fn query_watchlist_stocks(
    State(state): State<AppState>,
    Json(payload): Json<WatchlistQueryRequest>,
) -> Result<Json<WatchlistQueryResponse>, AppError> {
    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 查询观察表股票
    let results = stock_watchlist_query::query_watchlist_stocks(
        &mut conn,
        &payload.plate_codes,
        payload.change_pct_min.as_ref(),
        payload.change_pct_max.as_ref(),
        payload.volume_ratio_min.as_ref(),
        payload.volume_ratio_max.as_ref(),
        payload.turnover_rate_min.as_ref(),
        payload.turnover_rate_max.as_ref(),
        payload.bid_ask_ratio_min.as_ref(),
        payload.bid_ask_ratio_max.as_ref(),
        payload.main_force_inflow_min.as_ref(),
        payload.main_force_inflow_max.as_ref(),
        payload.stock_code_filter.as_deref(),
    )
    .map_err(|e| {
        tracing::error!("Failed to query watchlist stocks: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            WatchlistQueryItem {
                stock_code: r.stock_code,
                stock_name: r.stock_name,
                latest_price: r.latest_price,
                close_price: r.close_price,
                change_pct: r.change_pct,
                volume_ratio: r.volume_ratio,
                turnover_rate: r.turnover_rate,
                bid_ask_ratio: r.bid_ask_ratio,
                main_force_inflow: r.main_force_inflow,
                created_at: r.created_at,
                plates,
            }
        })
        .collect();

    Ok(Json(WatchlistQueryResponse { data, total }))
}

/// 查询股票明细（时间序列）
pub async fn query_stock_detail(
    State(state): State<AppState>,
    Json(payload): Json<WatchlistDetailRequest>,
) -> Result<Json<WatchlistDetailResponse>, AppError> {
    // 验证股票代码
    if payload.stock_code.trim().is_empty() {
        return Err(AppError::BadRequest("stock_code is required".to_string()));
    }

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 查询股票明细
    let results = stock_watchlist_query::query_stock_snapshot_detail(
        &mut conn,
        &payload.stock_code,
    )
    .map_err(|e| {
        tracing::error!("Failed to query stock detail: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            WatchlistDetailItem {
                stock_code: r.stock_code,
                stock_name: r.stock_name,
                latest_price: r.latest_price,
                close_price: r.close_price,
                change_pct: r.change_pct,
                volume_ratio: r.volume_ratio,
                turnover_rate: r.turnover_rate,
                bid_ask_ratio: r.bid_ask_ratio,
                main_force_inflow: r.main_force_inflow,
                created_at: r.created_at,
                plates,
            }
        })
        .collect();

    Ok(Json(WatchlistDetailResponse { data, total }))
}

/// 查询股票 K 线数据
pub async fn query_stock_kline(
    State(state): State<AppState>,
    Json(payload): Json<WatchlistKlineRequest>,
) -> Result<Json<WatchlistKlineResponse>, AppError> {
    // 验证股票代码
    if payload.stock_code.trim().is_empty() {
        return Err(AppError::BadRequest("stock_code is required".to_string()));
    }

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 查找首次出现日期
    let start_date = stock_watchlist_query::find_first_occurrence_date(
        &mut conn,
        &payload.stock_code,
    )
    .map_err(|e| {
        tracing::error!("Failed to find first occurrence date: {}", e);
        AppError::InternalServerError
    })?;

    // 如果没有找到首次出现日期，返回空结果
    let start_date = match start_date {
        Some(date) => date,
        None => {
            return Ok(Json(WatchlistKlineResponse {
                data: vec![],
                total: 0,
                start_date: None,
                end_date: Utc::now().date_naive(),
            }));
        }
    };

    // 结束日期为当前日期
    let end_date = Utc::now().date_naive();

    // 查询 K 线数据
    let results = stock_watchlist_query::query_stock_kline_range(
        &mut conn,
        &payload.stock_code,
        start_date,
        end_date,
    )
    .map_err(|e| {
        tracing::error!("Failed to query stock kline: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| WatchlistKlineItem {
            stock_code: r.stock_code,
            trade_date: r.trade_date,
            open_price: r.open_price,
            high_price: r.high_price,
            low_price: r.low_price,
            close_price: r.close_price,
            volume: r.volume,
            amount: r.amount,
        })
        .collect();

    Ok(Json(WatchlistKlineResponse {
        data,
        total,
        start_date: Some(start_date),
        end_date,
    }))
}

/// 补齐观察表K线数据
pub async fn fill_watchlist_klines(
    State(state): State<AppState>,
    Json(_payload): Json<WatchlistFillKlineRequest>,
) -> Result<Json<WatchlistFillKlineResponse>, AppError> {
    // 1. 获取所有观察表中的股票
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;
    
    let watchlist_items = stock_watchlist::list_all(&mut conn)
        .map_err(|e| {
            tracing::error!("Failed to list watchlist stocks: {}", e);
            AppError::InternalServerError
        })?;
    
    let stock_codes: Vec<String> = watchlist_items
        .into_iter()
        .map(|item| item.stock_code)
        .collect();
    
    if stock_codes.is_empty() {
        return Ok(Json(WatchlistFillKlineResponse {
            total_stocks: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            stock_details: Vec::new(),
        }));
    }
    
    // 2. 创建HTTP客户端
    let client = http_client::create_em_client()
        .map_err(|_| AppError::InternalServerError)?;
    
    // 3. 并发处理每个股票
    const HTTP_CONCURRENCY: usize = 10;
    const DB_CONCURRENCY: usize = 5;
    
    let http_semaphore = Arc::new(Semaphore::new(HTTP_CONCURRENCY));
    let db_semaphore = Arc::new(Semaphore::new(DB_CONCURRENCY));
    let mut join_set = JoinSet::new();
    
    for stock_code in stock_codes.iter().cloned() {
        let pool = state.db_pool.clone();
        let client = client.clone();
        let http_sem = http_semaphore.clone();
        let db_sem = db_semaphore.clone();
        
        join_set.spawn(async move {
            fill_single_stock_klines(pool, client, stock_code, http_sem, db_sem).await
        });
    }
    
    // 4. 收集结果
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut stock_details = Vec::new();
    
    while let Some(res) = join_set.join_next().await {
        match res {
            Ok(StockFillOutcome::Success(detail)) => {
                success_count += 1;
                stock_details.push(detail);
            }
            Ok(StockFillOutcome::Skipped(detail)) => {
                skipped_count += 1;
                stock_details.push(detail);
            }
            Ok(StockFillOutcome::Failed(detail)) => {
                failed_count += 1;
                stock_details.push(detail);
            }
            Err(e) => {
                failed_count += 1;
                tracing::error!("Task join error: {}", e);
                stock_details.push(StockFillKlineDetail {
                    stock_code: "unknown".to_string(),
                    imported_count: 0,
                    success: false,
                    error: Some(format!("Task join error: {e}")),
                });
            }
        }
    }
    
    Ok(Json(WatchlistFillKlineResponse {
        total_stocks: stock_codes.len(),
        success_count,
        failed_count,
        skipped_count,
        stock_details,
    }))
}

#[derive(Debug)]
enum StockFillOutcome {
    Success(StockFillKlineDetail),
    Skipped(StockFillKlineDetail),
    Failed(StockFillKlineDetail),
}

async fn fill_single_stock_klines(
    db_pool: crate::app::DbPool,
    client: reqwest::Client,
    stock_code: String,
    http_semaphore: Arc<Semaphore>,
    db_semaphore: Arc<Semaphore>,
) -> StockFillOutcome {
    // 1. 查询 stock_snapshots 中的日期范围
    let date_range = {
        let _db_permit = match db_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                return StockFillOutcome::Failed(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some("DB semaphore closed".to_string()),
                });
            }
        };
        let mut conn = match db_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                return StockFillOutcome::Failed(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some(format!("Failed to get DB connection: {e}")),
                });
            }
        };
        
        match stock_watchlist_query::find_snapshot_date_range(&mut conn, &stock_code) {
            Ok(Some(range)) => Some(range),
            Ok(None) => {
                // 没有快照数据，跳过
                return StockFillOutcome::Skipped(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: true,
                    error: Some("No snapshot data found".to_string()),
                });
            }
            Err(e) => {
                tracing::error!("Failed to query date range for {}: {}", stock_code, e);
                return StockFillOutcome::Failed(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some(format!("Failed to query date range: {e}")),
                });
            }
        }
    };
    
    let (start_date, end_date) = match date_range {
        Some(range) => range,
        None => {
            return StockFillOutcome::Skipped(StockFillKlineDetail {
                stock_code: stock_code.clone(),
                imported_count: 0,
                success: true,
                error: Some("No date range found".to_string()),
            });
        }
    };
    
    // 2. 查询已存在的K线日期
    let existing_dates = {
        let _db_permit = match db_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                return StockFillOutcome::Failed(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some("DB semaphore closed".to_string()),
                });
            }
        };
        let mut conn = match db_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                return StockFillOutcome::Failed(StockFillKlineDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some(format!("Failed to get DB connection: {e}")),
                });
            }
        };
        
        match stock_watchlist_query::find_existing_kline_dates(&mut conn, &stock_code, start_date, end_date) {
            Ok(dates) => dates,
            Err(e) => {
                tracing::warn!("Failed to query existing dates for {}: {}", stock_code, e);
                Vec::new()
            }
        }
    };
    
    let existing_dates_set: std::collections::HashSet<NaiveDate> = existing_dates.into_iter().collect();
    
    // 3. 计算需要补齐的日期范围（生成所有日期，排除已存在的）
    let mut dates_to_fill = Vec::new();
    let mut current_date = start_date;
    while current_date <= end_date {
        if !existing_dates_set.contains(&current_date) {
            dates_to_fill.push(current_date);
        }
        current_date = match current_date.succ_opt() {
            Some(date) => date,
            None => break,
        };
    }
    
    if dates_to_fill.is_empty() {
        return StockFillOutcome::Skipped(StockFillKlineDetail {
            stock_code: stock_code.clone(),
            imported_count: 0,
            success: true,
            error: Some("All dates already exist".to_string()),
        });
    }
    
    // 4. 按日期范围批量获取K线数据（东方财富API支持日期范围查询）
    let start_date_str = start_date.format("%Y%m%d").to_string();
    let end_date_str = end_date.format("%Y%m%d").to_string();
    
    let _http_permit = match http_semaphore.acquire_owned().await {
        Ok(permit) => permit,
        Err(_) => {
            return StockFillOutcome::Failed(StockFillKlineDetail {
                stock_code: stock_code.clone(),
                imported_count: 0,
                success: false,
                error: Some("HTTP semaphore closed".to_string()),
            });
        }
    };
    
    // 5. 获取K线数据
    let kline_result = match kline_service::fetch_and_parse_kline_data(
        &client,
        &stock_code,
        &start_date_str,
        &end_date_str,
    ).await {
        Ok(result) => result,
        Err(e) => {
            return StockFillOutcome::Failed(StockFillKlineDetail {
                stock_code: stock_code.clone(),
                imported_count: 0,
                success: false,
                error: Some(format!("Failed to fetch kline data: {e}")),
            });
        }
    };
    
    // 6. 过滤出需要插入的数据（排除已存在的日期）
    let klines_to_insert: Vec<_> = kline_result.parsed
        .into_iter()
        .filter(|kline| !existing_dates_set.contains(&kline.trade_date))
        .collect();
    
    if klines_to_insert.is_empty() {
        return StockFillOutcome::Skipped(StockFillKlineDetail {
            stock_code: stock_code.clone(),
            imported_count: 0,
            success: true,
            error: Some("No new data to insert".to_string()),
        });
    }
    
    // 7. 批量插入数据库
    let mut imported_count = 0;
    let mut last_error = None;
    
    for kline_data in klines_to_insert {
        let _db_permit = match db_semaphore.clone().acquire_owned().await {
            Ok(permit) => permit,
            Err(_) => {
                last_error = Some("DB semaphore closed".to_string());
                break;
            }
        };
        let mut conn = match db_pool.get() {
            Ok(conn) => conn,
            Err(e) => {
                last_error = Some(format!("Failed to get DB connection: {e}"));
                break;
            }
        };
        
        match daily_kline::create(&mut conn, &kline_data) {
            Ok(_) => imported_count += 1,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                // 重复数据，忽略（可能并发插入导致）
            }
            Err(e) => {
                tracing::warn!("Failed to insert kline for {} on {}: {}", 
                    stock_code, kline_data.trade_date, e);
                if last_error.is_none() {
                    last_error = Some(format!("Insert error: {e}"));
                }
            }
        }
    }
    
    if imported_count > 0 {
        StockFillOutcome::Success(StockFillKlineDetail {
            stock_code,
            imported_count,
            success: true,
            error: last_error,
        })
    } else if last_error.is_some() {
        StockFillOutcome::Failed(StockFillKlineDetail {
            stock_code,
            imported_count: 0,
            success: false,
            error: last_error,
        })
    } else {
        StockFillOutcome::Skipped(StockFillKlineDetail {
            stock_code,
            imported_count: 0,
            success: true,
            error: Some("No data inserted (all duplicates)".to_string()),
        })
    }
}
