use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateStockTable {
    pub stock_code: String,
    pub stock_name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateStockTableRequest {
    pub stock_code: Option<String>,
    pub stock_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StockTableResponse {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
