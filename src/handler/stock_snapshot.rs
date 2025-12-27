use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::result::Error as DieselError;
use serde::Serialize;

use crate::api_models::stock_snapshot::{CreateStockSnapshot, StockSnapshotResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewStockSnapshot;
use crate::repositories::stock_snapshot;

impl From<crate::models::StockSnapshot> for StockSnapshotResponse {
    fn from(s: crate::models::StockSnapshot) -> Self {
        Self {
            id: s.id,
            request_id: s.request_id,
            stock_code: s.stock_code,
            stock_name: s.stock_name,
            latest_price: s.latest_price,
            change_pct: s.change_pct,
            volume_ratio: s.volume_ratio,
            turnover_rate: s.turnover_rate,
            bid_ask_ratio: s.bid_ask_ratio,
            main_force_inflow: s.main_force_inflow,
            created_at: s.created_at,
        }
    }
}

pub async fn get_stock_snapshot(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<StockSnapshotResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = stock_snapshot::find_by_id(&mut conn, id).map_err(map_err)?;
    Ok(Json(found.into()))
}

#[derive(Serialize)]
pub struct InsertResponse {
    id: i32,
}

pub async fn create_stock_snapshot(
    State(state): State<AppState>,
    Json(payload): Json<CreateStockSnapshot>,
) -> Result<(StatusCode, Json<InsertResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_rec = NewStockSnapshot {
        request_id: payload.request_id,
        stock_code: payload.stock_code,
        stock_name: payload.stock_name,
        latest_price: payload.latest_price,
        change_pct: payload.change_pct,
        volume_ratio: payload.volume_ratio,
        turnover_rate: payload.turnover_rate,
        bid_ask_ratio: payload.bid_ask_ratio,
        main_force_inflow: payload.main_force_inflow,
    };
    let new_id = stock_snapshot::create(&mut conn, &new_rec).map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(InsertResponse { id: new_id })))
}

pub async fn delete_stock_snapshot(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = stock_snapshot::delete_by_id(&mut conn, id).map_err(map_err)?;
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

