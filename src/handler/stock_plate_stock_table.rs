use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use diesel::result::Error as DieselError;

use crate::api_models::stock_plate_stock_table::{
    CreateStockPlateStockTable, StockPlateStockItem, StockPlateStockQuery, StockPlateStockQueryResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewStockPlateStockTable;
use crate::repositories::stock_plate_stock_table;

pub async fn create_stock_plate_stock_table(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockPlateStockTable>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_rel = NewStockPlateStockTable {
        plate_id: payload.plate_id,
        stock_table_id: payload.stock_table_id,
    };
    stock_plate_stock_table::create(&mut conn, &new_rel).map_err(map_err)?;
    Ok(StatusCode::CREATED)
}

pub async fn query_stock_plate_stocks(
    State(state): State<AppState>,
    Query(params): Query<StockPlateStockQuery>,
) -> Result<Json<StockPlateStockQueryResponse>, AppError> {
    if params.page < 1 {
        return Err(AppError::BadRequest("page must be greater than 0".to_string()));
    }
    if params.page_size < 1 || params.page_size > 100 {
        return Err(AppError::BadRequest("page_size must be between 1 and 100".to_string()));
    }

    let plate_name_filter = params
        .plate_name
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty());

    let offset = (params.page - 1) * params.page_size;
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;

    let total = stock_plate_stock_table::count_plate_stocks(&mut conn, plate_name_filter)
        .map_err(|_| AppError::InternalServerError)?;

    let results = stock_plate_stock_table::query_plate_stocks(
        &mut conn,
        plate_name_filter,
        params.page_size,
        offset,
    )
    .map_err(|_| AppError::InternalServerError)?;

    let data = results
        .into_iter()
        .map(|r| StockPlateStockItem {
            plate_id: r.plate_id,
            plate_name: r.plate_name,
            stock_table_id: r.stock_table_id,
            stock_code: r.stock_code,
            stock_name: r.stock_name,
        })
        .collect();

    let total_pages = if total == 0 {
        0
    } else {
        (total + params.page_size - 1) / params.page_size
    };

    Ok(Json(StockPlateStockQueryResponse {
        data,
        total,
        page: params.page,
        page_size: params.page_size,
        total_pages,
    }))
}

pub async fn delete_stock_plate_stock_table(
    State(state): State<AppState>,
    Path((plate_id, stock_table_id)): Path<(i32, i32)>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_plate_stock_table::delete_by_pk(&mut conn, plate_id, stock_table_id)
        .map_err(map_err)?;
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
