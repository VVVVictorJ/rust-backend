use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

#[derive(Debug, Deserialize)]
pub struct CreateDailyKline {
    pub stock_code: String,
    pub trade_date: NaiveDate,
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: i64,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct DailyKlineResponse {
    pub stock_code: String,
    pub trade_date: NaiveDate,
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: i64,
    pub amount: BigDecimal,
}

