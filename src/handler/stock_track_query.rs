use axum::{
    extract::State,
    Json,
};
use chrono::NaiveDate;
use serde_json;

use crate::api_models::stock_trade_date_query::PlateInfo;
use crate::api_models::stock_track_query::{
    TrackQueryRequest, TrackQueryItem, TrackQueryResponse, OccurrenceStats,
    TrackDetailRequest, TrackDetailItem, TrackDetailResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_track_query;

/// 生成追踪标签
/// 14天≥3次，7天≥2次，3天≥2次
fn generate_tag(days_3: i32, days_7: i32, days_14: i32, min_occurrences_14: i32) -> String {
    let mut tags = Vec::new();
    
    if days_14 >= min_occurrences_14 {
        tags.push(format!("14天{days_14}次"));
    }
    if days_7 >= 2 {
        tags.push(format!("7天{days_7}次"));
    }
    if days_3 >= 2 {
        tags.push(format!("3天{days_3}次"));
    }
    
    if tags.is_empty() {
        String::new()
    } else {
        tags.join(" | ")
    }
}

/// 查询追踪股票列表
pub async fn query_tracked_stocks(
    State(state): State<AppState>,
    Json(payload): Json<TrackQueryRequest>,
) -> Result<Json<TrackQueryResponse>, AppError> {
    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

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
    let results = stock_track_query::query_tracked_stocks_by_date(
        &mut conn,
        trade_date,
        payload.min_occurrences,
    )
    .map_err(|e| {
        tracing::error!("Failed to query tracked stocks: {}", e);
        AppError::InternalServerError
    })?;

    let total = results.len() as i64;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();
            let tag = generate_tag(r.days_3_count, r.days_7_count, r.days_14_count, payload.min_occurrences);

            TrackQueryItem {
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
                occurrence_stats: OccurrenceStats {
                    days_3: r.days_3_count,
                    days_7: r.days_7_count,
                    days_14: r.days_14_count,
                },
                tag,
                plates,
            }
        })
        .collect();

    Ok(Json(TrackQueryResponse { data, total }))
}

/// 查询股票追踪明细
pub async fn query_stock_track_detail(
    State(state): State<AppState>,
    Json(payload): Json<TrackDetailRequest>,
) -> Result<Json<TrackDetailResponse>, AppError> {
    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    // 验证追踪天数参数
    if ![3, 7, 14].contains(&payload.track_days) {
        return Err(AppError::BadRequest("track_days must be 3, 7, or 14".to_string()));
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

    // 查询股票追踪明细
    let results = stock_track_query::query_stock_track_detail(
        &mut conn,
        &payload.stock_code,
        trade_date,
        payload.track_days,
    )
    .map_err(|e| {
        tracing::error!("Failed to query stock track detail: {}", e);
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
