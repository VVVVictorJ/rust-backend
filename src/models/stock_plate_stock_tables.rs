use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::stock_plate_stock_table;

#[allow(dead_code)]
#[derive(Queryable, Debug, Clone)]
#[diesel(belongs_to(crate::models::stock_plates::StockPlate, foreign_key = plate_id))]
#[diesel(belongs_to(crate::models::stock_tables::StockTable, foreign_key = stock_table_id))]
pub struct StockPlateStockTable {
    pub plate_id: i32,
    pub stock_table_id: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_plate_stock_table)]
pub struct NewStockPlateStockTable {
    pub plate_id: i32,
    pub stock_table_id: i32,
}
