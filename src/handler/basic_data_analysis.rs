use axum::{extract::State, Json};
use chrono::NaiveDate;

use crate::api_models::basic_data_analysis::{
    PlateStatisticsItem, PlateStatisticsRequest, PlateStatisticsResponse, PlateStockItem,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::basic_data_analysis;

pub async fn query_plate_statistics(
    State(state): State<AppState>,
    Json(payload): Json<PlateStatisticsRequest>,
) -> Result<Json<PlateStatisticsResponse>, AppError> {
    let trade_date = NaiveDate::parse_from_str(&payload.trade_date, "%Y-%m-%d").map_err(|_| {
        AppError::BadRequest("Invalid date format, expected YYYY-MM-DD".to_string())
    })?;

    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let summary = basic_data_analysis::query_plate_statistics_summary(&mut conn, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to query plate statistics summary: {}", e);
            AppError::InternalServerError
        })?;

    let data = basic_data_analysis::query_plate_statistics(&mut conn, trade_date)
        .map_err(|e| {
            tracing::error!("Failed to query plate statistics: {}", e);
            AppError::InternalServerError
        })?
        .into_iter()
        .map(|item| {
            let stocks: Vec<PlateStockItem> =
                serde_json::from_value(item.stocks).unwrap_or_default();
            PlateStatisticsItem {
                plate_code: item.plate_code,
                plate_name: item.plate_name,
                stock_count: item.stock_count,
                stocks,
            }
        })
        .collect::<Vec<_>>();

    let unclassified_count = summary.total_stock_count - summary.classified_stock_count;

    Ok(Json(PlateStatisticsResponse {
        trade_date: payload.trade_date,
        total_stock_count: summary.total_stock_count,
        classified_stock_count: summary.classified_stock_count,
        unclassified_count,
        plate_count: data.len() as i64,
        data,
    }))
}
