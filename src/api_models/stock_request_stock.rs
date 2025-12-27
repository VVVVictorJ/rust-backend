use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CreateStockRequestStock {
    pub request_id: i32,
    pub stock_code: String,
}

#[derive(Debug, serde::Serialize)]
pub struct StockRequestStockResponse {
    pub request_id: i32,
    pub stock_code: String,
}

