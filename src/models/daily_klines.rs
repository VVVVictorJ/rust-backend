use chrono::NaiveDate;
use diesel::prelude::*;
use bigdecimal::BigDecimal;

use crate::schema::daily_klines;

#[derive(Queryable, Debug, Clone)]
pub struct DailyKline {
    pub stock_code: String,
    pub trade_date: NaiveDate,
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: i64,
    pub amount: BigDecimal,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = daily_klines)]
pub struct NewDailyKline {
    pub stock_code: String,
    pub trade_date: NaiveDate,
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: i64,
    pub amount: BigDecimal,
}

