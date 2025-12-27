use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use diesel::result::Error as DieselError;
use serde::Serialize;

use crate::api_models::profit_analysis::{CreateProfitAnalysis, ProfitAnalysisResponse};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::NewProfitAnalysis;
use crate::repositories::profit_analysis;

impl From<crate::models::ProfitAnalysis> for ProfitAnalysisResponse {
    fn from(p: crate::models::ProfitAnalysis) -> Self {
        Self {
            id: p.id,
            snapshot_id: p.snapshot_id,
            strategy_name: p.strategy_name,
            profit_rate: p.profit_rate,
            analysis_time: p.analysis_time,
        }
    }
}

pub async fn get_profit_analysis(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<ProfitAnalysisResponse>, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let found = profit_analysis::find_by_id(&mut conn, id).map_err(map_err)?;
    Ok(Json(found.into()))
}

#[derive(Serialize)]
pub struct InsertResponse {
    id: i32,
}

pub async fn create_profit_analysis(
    State(state): State<AppState>,
    Json(payload): Json<CreateProfitAnalysis>,
) -> Result<(StatusCode, Json<InsertResponse>), AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let new_rec = NewProfitAnalysis {
        snapshot_id: payload.snapshot_id,
        strategy_name: payload.strategy_name,
        profit_rate: payload.profit_rate,
    };
    let new_id = profit_analysis::create(&mut conn, &new_rec).map_err(map_err)?;
    Ok((StatusCode::CREATED, Json(InsertResponse { id: new_id })))
}

pub async fn delete_profit_analysis(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state.db_pool.get().map_err(|_| AppError::InternalServerError)?;
    let affected = profit_analysis::delete_by_id(&mut conn, id).map_err(map_err)?;
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

