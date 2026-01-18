use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateStockPlate {
    pub plate_code: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStockPlateRequest {
    pub plate_code: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StockPlateResponse {
    pub id: i32,
    pub plate_code: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
