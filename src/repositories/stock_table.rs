use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::OptionalExtension;

use crate::models::{NewStockTable, StockTable, UpdateStockTable};
use crate::schema::stock_table::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_stock: &NewStockTable) -> Result<StockTable, diesel::result::Error> {
    diesel::insert_into(stock_table)
        .values(new_stock)
        .get_result(conn)
}

pub fn find_by_id(conn: &mut PgPoolConn, stock_id: i32) -> Result<StockTable, diesel::result::Error> {
    stock_table.find(stock_id).first(conn)
}

pub fn list_all(conn: &mut PgPoolConn) -> Result<Vec<StockTable>, diesel::result::Error> {
    stock_table.order(id.asc()).load(conn)
}

pub fn update_by_id(
    conn: &mut PgPoolConn,
    stock_id: i32,
    update_data: &UpdateStockTable,
) -> Result<StockTable, diesel::result::Error> {
    diesel::update(stock_table.find(stock_id))
        .set(update_data)
        .get_result(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, stock_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_table.find(stock_id)).execute(conn)
}

pub fn exists_by_code(conn: &mut PgPoolConn, code: &str) -> Result<bool, diesel::result::Error> {
    let existing = stock_table
        .filter(stock_code.eq(code))
        .select(id)
        .first::<i32>(conn)
        .optional()?;
    Ok(existing.is_some())
}
