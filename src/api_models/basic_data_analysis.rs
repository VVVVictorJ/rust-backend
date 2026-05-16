use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct PlateStatisticsRequest {
    pub trade_date: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlateStockItem {
    pub stock_code: String,
    pub stock_name: String,
}

#[derive(Debug, Serialize)]
pub struct PlateStatisticsItem {
    pub plate_code: String,
    pub plate_name: String,
    pub stock_count: i64,
    pub stocks: Vec<PlateStockItem>,
}

#[derive(Debug, Serialize)]
pub struct PlateStatisticsResponse {
    pub trade_date: String,
    pub total_stock_count: i64,
    pub classified_stock_count: i64,
    pub unclassified_count: i64,
    pub plate_count: i64,
    pub data: Vec<PlateStatisticsItem>,
}
