use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::{NewStockSnapshot, StockSnapshot};
use crate::schema::stock_snapshots::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn find_by_id(conn: &mut PgPoolConn, snapshot_id: i32) -> Result<StockSnapshot, diesel::result::Error> {
    stock_snapshots.find(snapshot_id).first(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, snapshot_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_snapshots.find(snapshot_id)).execute(conn)
}

pub fn create(conn: &mut PgPoolConn, new_rec: &NewStockSnapshot) -> Result<i32, diesel::result::Error> {
    diesel::insert_into(stock_snapshots)
        .values(new_rec)
        .returning(id)
        .get_result(conn)
}

