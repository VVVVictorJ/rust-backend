use axum::{
    extract::State,
    Json,
};
use chrono::NaiveDate;
use serde_json;

use crate::api_models::stock_trade_date_query::{
    PlateInfo, TradeDateQueryRequest, TradeDateQueryItem, TradeDateQueryResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::stock_trade_date_query;

/// 根据交易日期查询股票快照数据
pub async fn query_by_trade_date(
    State(state): State<AppState>,
    Json(payload): Json<TradeDateQueryRequest>,
) -> Result<Json<TradeDateQueryResponse>, AppError> {
    // 验证分页参数
    if payload.page < 1 {
        return Err(AppError::BadRequest("page must be greater than 0".to_string()));
    }
    if payload.page_size < 1 || payload.page_size > 100 {
        return Err(AppError::BadRequest("page_size must be between 1 and 100".to_string()));
    }

    // 解析交易日期
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d")
        .map_err(|_| AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string()))?;

    // 获取数据库连接
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    // 计算分页参数
    let offset = (payload.page - 1) * payload.page_size;

    // 查询总数
    let total = stock_trade_date_query::count_by_trade_date(&mut conn, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to count records: {}", e);
            AppError::InternalServerError
        })?;

    // 查询数据
    let results = stock_trade_date_query::query_by_trade_date(
        &mut conn,
        trade_date,
        payload.page_size,
        offset,
    )
    .map_err(|e| {
        tracing::error!("Failed to query data: {}", e);
        AppError::InternalServerError
    })?;

    // 转换结果
    let data = results
        .into_iter()
        .map(|r| {
            let plates: Vec<PlateInfo> = serde_json::from_value(r.plates).unwrap_or_default();

            TradeDateQueryItem {
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

    // 计算总页数
    let total_pages = if total == 0 {
        0
    } else {
        (total + payload.page_size - 1) / payload.page_size
    };

    Ok(Json(TradeDateQueryResponse {
        data,
        total,
        page: payload.page,
        page_size: payload.page_size,
        total_pages,
    }))
}

