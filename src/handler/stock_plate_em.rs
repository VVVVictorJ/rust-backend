use axum::extract::Query;
use axum::Json;

use crate::api_models::stock_plate_em::EmPlateResponse;
use crate::handler::error::AppError;
use crate::services::stock_plate_em::fetch_em_plate_list;
use crate::utils::http_client::create_em_client;

#[derive(Debug, serde::Deserialize)]
pub struct EmPlateQuery {
    pub stock_code: String,
}

pub async fn fetch_em_stock_plates(
    Query(query): Query<EmPlateQuery>,
) -> Result<Json<EmPlateResponse>, AppError> {
    let stock_code = query.stock_code.trim();
    if stock_code.is_empty() {
        return Err(AppError::BadRequest("stock_code is required".to_string()));
    }
    let client = create_em_client().map_err(|_| AppError::InternalServerError)?;
    let response = fetch_em_plate_list(&client, stock_code)
        .await
        .map_err(|_| AppError::InternalServerError)?;
    Ok(Json(response))
}
