use axum::{
    extract::State,
    Json,
};
use chrono::NaiveDate;
use serde_json;

use crate::api_models::dynamic_backtrack::{
    DynamicBacktrackRequest, DynamicBacktrackItem, DynamicBacktrackResponse,
    DynamicBacktrackDetailRequest, TrackDetailItem, TrackDetailResponse,
};
use crate::api_models::stock_trade_date_query::PlateInfo;
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::dynamic_backtrack;

/// 查询动态回溯股票列表
pub async fn query_dynamic_backtrack(
    State(state): State<AppState>,
    Json(payload): Json<DynamicBacktrackRequest>,
) -> Result<Json<DynamicBacktrackResponse>, AppError> {
    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    // 验证交易日数参数
    if payload.trade_days < 1 {
        return Err(AppError::BadRequest("trade_days must be greater than 0".to_string()));
    }

    // 验证最少出现次数参数
    if payload.min_occurrences < 1 {
        return Err(AppError::BadRequest("min_occurrences must be greater than 0".to_string()));
    }

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 查询满足条件的股票
    let results = dynamic_backtrack::query_dynamic_backtrack(
        &mut conn,
        trade_date,
        payload.trade_days,
        payload.min_occurrences,
    )
    .map_err(|e| {
        tracing::error!("Failed to query dynamic backtrack stocks: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            DynamicBacktrackItem {
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
                occurrence_count: r.occurrence_count,
                plates,
            }
        })
        .collect();

    Ok(Json(DynamicBacktrackResponse { data, total }))
}

/// 查询动态回溯股票明细
pub async fn query_dynamic_backtrack_detail(
    State(state): State<AppState>,
    Json(payload): Json<DynamicBacktrackDetailRequest>,
) -> Result<Json<TrackDetailResponse>, AppError> {
    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    // 验证交易日数参数
    if payload.trade_days < 1 {
        return Err(AppError::BadRequest("trade_days must be greater than 0".to_string()));
    }

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
    let results = dynamic_backtrack::query_dynamic_backtrack_detail(
        &mut conn,
        &payload.stock_code,
        trade_date,
        payload.trade_days,
    )
    .map_err(|e| {
        tracing::error!("Failed to query dynamic backtrack detail: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            TrackDetailItem {
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

    Ok(Json(TrackDetailResponse { data, total }))
}
