use axum::{extract::State, Json};

use crate::api_models::daily_kline::DailyKlineResponse;
use crate::api_models::monthly_kline::{MonthlyKlineQueryRequest, MonthlyKlineQueryResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewDailyKline;
use crate::services::kline_service;

fn new_daily_to_response(row: NewDailyKline) -> DailyKlineResponse {
    DailyKlineResponse {
        stock_code: row.stock_code,
        trade_date: row.trade_date,
        open_price: row.open_price,
        high_price: row.high_price,
        low_price: row.low_price,
        close_price: row.close_price,
        volume: row.volume,
        amount: row.amount,
    }
}

/// 按月 K（东方财富 `klt=103`）查询 K 序列，不写库。
pub async fn monthly_kline_query(
    State(_state): State<AppState>,
    Json(payload): Json<MonthlyKlineQueryRequest>,
) -> Result<Json<MonthlyKlineQueryResponse>, AppError> {
    let code = payload.stock_code.trim();
    if code.is_empty() {
        return Err(AppError::BadRequest("stock_code is required".to_string()));
    }

    let result = kline_service::fetch_and_parse_monthly_kline_via_proxy_only(code)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to fetch monthly klines: {e}")))?;

    let parse_errors = if result.errors.is_empty() {
        None
    } else {
        Some(result.errors)
    };

    let klines: Vec<DailyKlineResponse> = result
        .parsed
        .into_iter()
        .map(new_daily_to_response)
        .collect();

    Ok(Json(MonthlyKlineQueryResponse {
        stock_code: result.stock_code,
        stock_name: result.stock_name,
        total_count: result.total,
        parse_errors,
        klines,
    }))
}
