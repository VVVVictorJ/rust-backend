use axum::{
    extract::State,
    Json,
};
use chrono::{NaiveDate, Utc};
use serde_json;

use crate::api_models::stock_trade_date_query::PlateInfo;
use crate::api_models::stock_watchlist_query::{
    WatchlistQueryRequest, WatchlistQueryItem, WatchlistQueryResponse,
    WatchlistDetailRequest, WatchlistDetailItem, WatchlistDetailResponse,
    WatchlistKlineRequest, WatchlistKlineItem, WatchlistKlineResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_watchlist_query;

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
