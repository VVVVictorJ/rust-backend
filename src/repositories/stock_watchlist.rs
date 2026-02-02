use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::OptionalExtension;

use crate::models::{NewStockWatchlist, StockWatchlist, UpdateStockWatchlist};
use crate::schema::stock_watchlist::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_item: &NewStockWatchlist) -> Result<StockWatchlist, diesel::result::Error> {
    diesel::insert_into(stock_watchlist)
        .values(new_item)
        .get_result(conn)
}

pub fn find_by_code(
    conn: &mut PgPoolConn,
    code: &str,
) -> Result<Option<StockWatchlist>, diesel::result::Error> {
    stock_watchlist
        .filter(stock_code.eq(code))
        .first::<StockWatchlist>(conn)
        .optional()
}

pub fn list_all(conn: &mut PgPoolConn) -> Result<Vec<StockWatchlist>, diesel::result::Error> {
    stock_watchlist.order(created_at.desc()).load(conn)
}

pub fn delete_by_code(conn: &mut PgPoolConn, code: &str) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_watchlist.filter(stock_code.eq(code))).execute(conn)
}

pub fn exists_by_code(conn: &mut PgPoolConn, code: &str) -> Result<bool, diesel::result::Error> {
    let existing = stock_watchlist
        .filter(stock_code.eq(code))
        .select(id)
        .first::<i32>(conn)
        .optional()?;
    Ok(existing.is_some())
}

#[allow(dead_code)]
pub fn update_by_code(
    conn: &mut PgPoolConn,
    code: &str,
    update_data: &UpdateStockWatchlist,
) -> Result<StockWatchlist, diesel::result::Error> {
    diesel::update(stock_watchlist.filter(stock_code.eq(code)))
        .set(update_data)
        .get_result(conn)
}
