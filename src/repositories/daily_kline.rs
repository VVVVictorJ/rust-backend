use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::{DailyKline, NewDailyKline};
use crate::schema::daily_klines::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_rec: &NewDailyKline) -> Result<DailyKline, diesel::result::Error> {
    diesel::insert_into(daily_klines)
        .values(new_rec)
        .get_result(conn)
}

pub fn find_by_pk(conn: &mut PgPoolConn, code: &str, date: NaiveDate) -> Result<DailyKline, diesel::result::Error> {
    daily_klines
        .filter(stock_code.eq(code))
        .filter(trade_date.eq(date))
        .first(conn)
}

pub fn delete_by_pk(conn: &mut PgPoolConn, code: &str, date: NaiveDate) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        daily_klines
            .filter(stock_code.eq(code))
            .filter(trade_date.eq(date)),
    )
    .execute(conn)
}

/// 检查指定股票在指定日期是否已有K线数据
pub fn exists(conn: &mut PgPoolConn, code: &str, date: NaiveDate) -> Result<bool, diesel::result::Error> {
    use diesel::dsl::exists;
    use diesel::select;
    
    select(exists(
        daily_klines
            .filter(stock_code.eq(code))
            .filter(trade_date.eq(date))
    ))
    .get_result(conn)
}

/// 查询指定日期之前最近的交易日期（处理节假日）
pub fn find_previous_trade_date(conn: &mut PgPoolConn, date: NaiveDate) -> Result<Option<NaiveDate>, diesel::result::Error> {
    daily_klines
        .select(trade_date)
        .filter(trade_date.lt(date))
        .order(trade_date.desc())
        .first::<NaiveDate>(conn)
        .optional()
}

