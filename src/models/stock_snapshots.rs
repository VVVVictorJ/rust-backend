use chrono::{DateTime, Utc};
use diesel::prelude::*;
use bigdecimal::BigDecimal;

use crate::schema::stock_snapshots;

#[derive(Queryable, Debug, Clone)]
#[diesel(belongs_to(crate::models::stock_requests::StockRequest, foreign_key = request_id))]
pub struct StockSnapshot {
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

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_snapshots)]
pub struct NewStockSnapshot {
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

