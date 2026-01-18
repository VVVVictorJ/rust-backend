use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateStockPlateStockTable {
    pub plate_id: i32,
    pub stock_table_id: i32,
}

#[derive(Debug, Deserialize)]
pub struct StockPlateStockQuery {
    #[serde(default)]
    pub plate_name: Option<String>,
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Serialize)]
pub struct StockPlateStockItem {
    pub plate_id: i32,
    pub plate_name: String,
    pub stock_table_id: i32,
    pub stock_code: String,
    pub stock_name: String,
}

#[derive(Debug, Serialize)]
pub struct StockPlateStockQueryResponse {
    pub data: Vec<StockPlateStockItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
}
