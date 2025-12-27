use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use uuid::Uuid;
use crate::schema::stock_requests;

#[derive(Queryable, Debug, Clone)]
pub struct StockRequest {
    pub id: i32,
    pub request_uuid: Uuid,
    pub request_time: DateTime<Utc>,
    pub strategy_name: Option<String>,
    pub time_range_start: Option<NaiveDate>,
    pub time_range_end: Option<NaiveDate>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_requests)]
pub struct NewStockRequest {
    pub strategy_name: Option<String>,
    pub time_range_start: Option<NaiveDate>,
    pub time_range_end: Option<NaiveDate>,
}

