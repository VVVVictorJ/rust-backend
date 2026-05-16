use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct StockSnapshotResponse {
    pub id: i32,
    pub request_id: i32,
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub change_pct: BigDecimal,
    pub volume_ratio: BigDecimal,
    pub turnover_rate: BigDecimal,
    pub bid_ask_ratio: BigDecimal,
    pub main_force_inflow: BigDecimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStockSnapshot {
    pub request_id: i32,
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub change_pct: BigDecimal,
    pub volume_ratio: BigDecimal,
    pub turnover_rate: BigDecimal,
    pub bid_ask_ratio: BigDecimal,
    pub main_force_inflow: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct TodayStockCodesResponse {
    pub count: usize,
    pub stock_codes: Vec<String>,
}
