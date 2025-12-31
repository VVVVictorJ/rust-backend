use axum::{
    extract::State,
    Json,
};
use chrono::NaiveDate;

use crate::api_models::stock_price_compare::{
    PriceCompareRequest, PriceCompareItem, PriceCompareResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::{daily_kline, stock_price_compare};

/// 根据交易日期查询价格对比数据
pub async fn query_price_compare(
    State(state): State<AppState>,
    Json(payload): Json<PriceCompareRequest>,
) -> Result<Json<PriceCompareResponse>, AppError> {
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

    // 查询前一个交易日（处理节假日）
    let snapshot_date = daily_kline::find_previous_trade_date(&mut conn, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to find previous trade date: {}", e);
            AppError::InternalServerError
        })?;

    // 如果找不到前一个交易日，返回空结果
    if snapshot_date.is_none() {
        return Ok(Json(PriceCompareResponse {
            data: vec![],
            total: 0,
            page: payload.page,
            page_size: payload.page_size,
            total_pages: 0,
            snapshot_date: None,
            trade_date: Some(trade_date),
        }));
    }

    let snapshot_date = snapshot_date.unwrap();

    // 计算分页参数
    let offset = (payload.page - 1) * payload.page_size;

    // 查询总数
    let total = stock_price_compare::count_price_compare(&mut conn, snapshot_date, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to count records: {}", e);
            AppError::InternalServerError
        })?;

    // 查询数据
    let results = stock_price_compare::query_price_compare(
        &mut conn,
        snapshot_date,
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
        .map(|r| PriceCompareItem {
            stock_code: r.stock_code,
            stock_name: r.stock_name,
            latest_price: r.latest_price,
            high_price: r.high_price,
            close_price: r.close_price,
            open_price: r.open_price,
            low_price: r.low_price,
            grade: r.grade,
            created_at: r.created_at,
        })
        .collect();

    // 计算总页数
    let total_pages = if total == 0 {
        0
    } else {
        (total + payload.page_size - 1) / payload.page_size
    };

    Ok(Json(PriceCompareResponse {
        data,
        total,
        page: payload.page,
        page_size: payload.page_size,
        total_pages,
        snapshot_date: Some(snapshot_date),
        trade_date: Some(trade_date),
    }))
}

