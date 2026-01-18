use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use diesel::result::Error as DieselError;

use crate::api_models::stock_plate::{CreateStockPlate, StockPlateResponse, UpdateStockPlateRequest};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::{NewStockPlate, UpdateStockPlate};
use crate::repositories::stock_plate;

impl From<crate::models::StockPlate> for StockPlateResponse {
    fn from(plate: crate::models::StockPlate) -> Self {
        Self {
            id: plate.id,
            plate_code: plate.plate_code,
            name: plate.name,
            created_at: plate.created_at,
            updated_at: plate.updated_at,
        }
    }
}

pub async fn create_stock_plate(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockPlate>,
) -> Result<(StatusCode, Json<StockPlateResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_plate = NewStockPlate {
        plate_code: payload.plate_code,
        name: payload.name,
    };
    let created = stock_plate::create(&mut conn, &new_plate).map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

pub async fn get_stock_plate(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<StockPlateResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = stock_plate::find_by_id(&mut conn, id).map_err(map_err)?;
    Ok(Json(found.into()))
}

pub async fn list_stock_plates(
    State(state): State<AppState>,
) -> Result<Json<Vec<StockPlateResponse>>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let items = stock_plate::list_all(&mut conn).map_err(map_err)?;
    let response: Vec<StockPlateResponse> = items.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

pub async fn update_stock_plate(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateStockPlateRequest>,
) -> Result<Json<StockPlateResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let update_data = UpdateStockPlate {
        plate_code: payload.plate_code,
        name: payload.name,
        updated_at: Some(Utc::now().naive_utc()),
    };
    let updated = stock_plate::update_by_id(&mut conn, id, &update_data).map_err(map_err)?;
    Ok(Json(updated.into()))
}

pub async fn delete_stock_plate(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_plate::delete_by_id(&mut conn, id).map_err(map_err)?;
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
