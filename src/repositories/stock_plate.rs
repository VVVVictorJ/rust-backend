use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::OptionalExtension;

use crate::models::{NewStockPlate, StockPlate, UpdateStockPlate};
use crate::schema::stock_plate::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(conn: &mut PgPoolConn, new_plate: &NewStockPlate) -> Result<StockPlate, diesel::result::Error> {
    diesel::insert_into(stock_plate)
        .values(new_plate)
        .get_result(conn)
}

pub fn find_by_id(conn: &mut PgPoolConn, plate_id: i32) -> Result<StockPlate, diesel::result::Error> {
    stock_plate.find(plate_id).first(conn)
}

pub fn list_all(conn: &mut PgPoolConn) -> Result<Vec<StockPlate>, diesel::result::Error> {
    stock_plate.order(id.asc()).load(conn)
}

pub fn find_by_plate_code(
    conn: &mut PgPoolConn,
    plate_code_val: &str,
) -> Result<Option<StockPlate>, diesel::result::Error> {
    stock_plate
        .filter(plate_code.eq(plate_code_val))
        .first::<StockPlate>(conn)
        .optional()
}

pub fn find_by_name(
    conn: &mut PgPoolConn,
    plate_name: &str,
) -> Result<Option<StockPlate>, diesel::result::Error> {
    stock_plate
        .filter(name.eq(plate_name))
        .first::<StockPlate>(conn)
        .optional()
}

pub fn update_by_id(
    conn: &mut PgPoolConn,
    plate_id: i32,
    update_data: &UpdateStockPlate,
) -> Result<StockPlate, diesel::result::Error> {
    diesel::update(stock_plate.find(plate_id))
        .set(update_data)
        .get_result(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, plate_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_plate.find(plate_id)).execute(conn)
}
