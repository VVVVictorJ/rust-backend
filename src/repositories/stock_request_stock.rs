use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::NewStockRequestStock;
use crate::models::StockRequestStock;
use crate::schema::stock_request_stocks::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_req: &NewStockRequestStock) -> Result<(), diesel::result::Error> {
    diesel::insert_into(stock_request_stocks)
        .values(new_req)
        .execute(conn)?;
    Ok(())
}

pub fn find_by_pk(conn: &mut PgPoolConn, req_id: i32, code: &str) -> Result<StockRequestStock, diesel::result::Error> {
    stock_request_stocks
        .filter(request_id.eq(req_id))
        .filter(stock_code.eq(code))
        .first(conn)
}

pub fn delete_by_pk(conn: &mut PgPoolConn, req_id: i32, code: &str) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        stock_request_stocks
            .filter(request_id.eq(req_id))
            .filter(stock_code.eq(code)),
    )
    .execute(conn)
}

