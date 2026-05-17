use axum::{extract::State, Json};

use crate::api_models::convertible_bond_query::ConvertibleBondQueryResponse;
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::services::convertible_bond_query::fetch_filtered_convertible_bonds;
use crate::utils::http_client::create_em_client;

pub async fn query_convertible_bonds(
    State(_state): State<AppState>,
) -> Result<Json<ConvertibleBondQueryResponse>, AppError> {
    let client = create_em_client().map_err(|_| AppError::InternalServerError)?;
    let data = fetch_filtered_convertible_bonds(&client).await.map_err(|err| {
        tracing::error!("query_convertible_bonds failed: {}", err);
        AppError::InternalServerError
    })?;

    Ok(Json(ConvertibleBondQueryResponse {
        total: data.len() as i64,
        data,
    }))
}
