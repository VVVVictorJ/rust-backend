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

