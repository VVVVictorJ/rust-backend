use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct StockRequestResponse {
    pub id: i32,
    pub request_uuid: Uuid,
    pub request_time: DateTime<Utc>,
    pub strategy_name: Option<String>,
    pub time_range_start: Option<NaiveDate>,
    pub time_range_end: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct CreateStockRequest {
    pub strategy_name: Option<String>,
    pub time_range_start: Option<NaiveDate>,
    pub time_range_end: Option<NaiveDate>,
}

