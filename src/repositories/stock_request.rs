use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::{NewStockRequest, StockRequest};
use crate::schema::stock_requests::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn find_by_id(conn: &mut PgPoolConn, req_id: i32) -> Result<StockRequest, diesel::result::Error> {
    stock_requests.find(req_id).first(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, req_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_requests.find(req_id)).execute(conn)
}

pub fn create(conn: &mut PgPoolConn, new_req: &NewStockRequest) -> Result<StockRequest, diesel::result::Error> {
    diesel::insert_into(stock_requests)
        .values(new_req)
        .get_result(conn)
}

