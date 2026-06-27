use axum::{
    extract::{Query, State},
    Json,
};

use crate::api_models::bagua::{
    AlmanacQuery, AlmanacResponse, HetuLookupCell, HetuLookupQuery, HetuLookupResponse,
    HetuSingleLookupResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::he_luo_lookup;
use crate::services::almanac;

const HETU_MATRIX: &str = "hetu";
const LUOSHU_STEM_MATRIX: &str = "luoshu_stem";
const LUOSHU_BRANCH_MATRIX: &str = "luoshu_branch";

async fn get_matrix_lookup(
    state: AppState,
    matrix_code: &str,
    params: HetuLookupQuery,
) -> Result<Json<serde_json::Value>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    if let (Some(row), Some(col)) = (&params.row, &params.col) {
        let cell = he_luo_lookup::find_cell(&mut conn, matrix_code, row, col).map_err(|e| {
            tracing::error!("Failed to query {matrix_code} lookup cell: {e}");
            AppError::InternalServerError
        })?;

        let cell = cell.ok_or(AppError::NotFound)?;
        return Ok(Json(
            serde_json::to_value(HetuSingleLookupResponse {
                row_key: cell.row_key,
                col_key: cell.col_key,
                value: cell.value,
            })
            .map_err(|_| AppError::InternalServerError)?,
        ));
    }

    let rows = he_luo_lookup::list_by_matrix(&mut conn, matrix_code).map_err(|e| {
        tracing::error!("Failed to list {matrix_code} lookup: {e}");
        AppError::InternalServerError
    })?;

    let cells = rows
        .into_iter()
        .map(|row| HetuLookupCell {
            row_key: row.row_key,
            col_key: row.col_key,
            value: row.value,
        })
        .collect();

    Ok(Json(
        serde_json::to_value(HetuLookupResponse {
            matrix_code: matrix_code.to_string(),
            cells,
        })
        .map_err(|_| AppError::InternalServerError)?,
    ))
}

pub async fn get_hetu_lookup(
    State(state): State<AppState>,
    Query(params): Query<HetuLookupQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    get_matrix_lookup(state, HETU_MATRIX, params).await
}

pub async fn get_luoshu_stem_lookup(
    State(state): State<AppState>,
    Query(params): Query<HetuLookupQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    get_matrix_lookup(state, LUOSHU_STEM_MATRIX, params).await
}

pub async fn get_luoshu_branch_lookup(
    State(state): State<AppState>,
    Query(params): Query<HetuLookupQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    get_matrix_lookup(state, LUOSHU_BRANCH_MATRIX, params).await
}

pub async fn get_almanac(
    Query(params): Query<AlmanacQuery>,
) -> Result<Json<AlmanacResponse>, AppError> {
    if params.year.is_empty() || params.month.is_empty() || params.day.is_empty() {
        return Err(AppError::BadRequest(
            "year, month, day are required".to_string(),
        ));
    }

    let result = almanac::fetch_almanac(&params.year, &params.month, &params.day).await?;
    Ok(Json(result))
}
