use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::NewStockRequestStock;
use crate::schema::stock_request_stocks::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_req: &NewStockRequestStock) -> Result<(), diesel::result::Error> {
    diesel::insert_into(stock_request_stocks)
        .values(new_req)
        .execute(conn)?;
    Ok(())
}

