use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::result::Error as DieselError;

use crate::api_models::stock_table::{CreateStockTable, StockTableResponse, UpdateStockTableRequest};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::{NewStockTable, UpdateStockTable};
use crate::repositories::stock_table;

impl From<crate::models::StockTable> for StockTableResponse {
    fn from(stock: crate::models::StockTable) -> Self {
        Self {
            id: stock.id,
            stock_code: stock.stock_code,
            stock_name: stock.stock_name,
            created_at: stock.created_at,
            updated_at: stock.updated_at,
        }
    }
}

pub async fn create_stock_table(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockTable>,
) -> Result<(StatusCode, Json<StockTableResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_stock = NewStockTable {
        stock_code: payload.stock_code,
        stock_name: payload.stock_name,
    };
    let created = stock_table::create(&mut conn, &new_stock).map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

pub async fn get_stock_table(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<StockTableResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = stock_table::find_by_id(&mut conn, id).map_err(map_err)?;
    Ok(Json(found.into()))
}

pub async fn list_stock_tables(
    State(state): State<AppState>,
) -> Result<Json<Vec<StockTableResponse>>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let items = stock_table::list_all(&mut conn).map_err(map_err)?;
    let response: Vec<StockTableResponse> = items.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn update_stock_table(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateStockTableRequest>,
) -> Result<Json<StockTableResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let update_data = UpdateStockTable {
        stock_code: payload.stock_code,
        stock_name: payload.stock_name,
        updated_at: Some(Utc::now().naive_utc()),
    };
    let updated = stock_table::update_by_id(&mut conn, id, &update_data).map_err(map_err)?;
    Ok(Json(updated.into()))
}

pub async fn delete_stock_table(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_table::delete_by_id(&mut conn, id).map_err(map_err)?;
    if affected == 0 {
        return Err(AppError::NotFound);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn map_err(err: DieselError) -> AppError {
    match err {
        DieselError::NotFound => AppError::NotFound,
        _ => AppError::InternalServerError,
    }
}
