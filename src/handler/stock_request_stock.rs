use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::result::Error as DieselError;

use crate::api_models::stock_request_stock::{CreateStockRequestStock, StockRequestStockResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewStockRequestStock;
use crate::repositories::stock_request_stock;

pub async fn create_stock_request_stock(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockRequestStock>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_rec = NewStockRequestStock {
        request_id: payload.request_id,
        stock_code: payload.stock_code,
    };
    stock_request_stock::create(&mut conn, &new_rec).map_err(map_err)?;
    Ok(StatusCode::CREATED)
}

pub async fn get_stock_request_stock(
    State(state): State<AppState>,
    Path((req_id, code)): Path<(i32, String)>,
) -> Result<Json<StockRequestStockResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = stock_request_stock::find_by_pk(&mut conn, req_id, &code).map_err(map_err)?;
    Ok(Json(StockRequestStockResponse {
        request_id: found.request_id,
        stock_code: found.stock_code,
    }))
}

pub async fn delete_stock_request_stock(
    State(state): State<AppState>,
    Path((req_id, code)): Path<(i32, String)>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_request_stock::delete_by_pk(&mut conn, req_id, &code).map_err(map_err)?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: DieselError) -> AppError {
    match err {
        DieselError::DatabaseError(_, info) => AppError::BadRequest(info.message().to_string()),
        _ => AppError::InternalServerError,
    }
}

