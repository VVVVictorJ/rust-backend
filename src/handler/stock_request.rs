use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use diesel::result::Error as DieselError;
use serde_json::json;

use crate::api_models::stock_request::{CreateStockRequest, StockRequestResponse};
use crate::app::AppState;
use crate::models::NewStockRequest;
use crate::repositories::stock_request;

#[derive(Debug)]
pub enum AppError {
    NotFound,
    InternalServerError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, Json(json!({"error": "not found"}))).into_response(),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "internal server error"})),
            )
                .into_response(),
        }
    }
}

pub async fn create_stock_request(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockRequest>,
) -> Result<Json<StockRequestResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_req = NewStockRequest {
        strategy_name: payload.strategy_name,
        time_range_start: payload.time_range_start,
        time_range_end: payload.time_range_end,
    };
    let created = stock_request::create(&mut conn, &new_req).map_err(map_diesel_error)?;
    Ok(Json(StockRequestResponse {
        id: created.id,
        request_uuid: created.request_uuid,
        request_time: created.request_time,
        strategy_name: created.strategy_name,
        time_range_start: created.time_range_start,
        time_range_end: created.time_range_end,
    }))
}

pub async fn get_stock_request(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<StockRequestResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = stock_request::find_by_id(&mut conn, id).map_err(map_diesel_error)?;
    Ok(Json(StockRequestResponse {
        id: found.id,
        request_uuid: found.request_uuid,
        request_time: found.request_time,
        strategy_name: found.strategy_name,
        time_range_start: found.time_range_start,
        time_range_end: found.time_range_end,
    }))
}

pub async fn delete_stock_request(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_request::delete_by_id(&mut conn, id).map_err(map_diesel_error)?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn map_diesel_error(err: DieselError) -> AppError {
    match err {
        DieselError::NotFound => AppError::NotFound,
        _ => AppError::InternalServerError,
    }
}

