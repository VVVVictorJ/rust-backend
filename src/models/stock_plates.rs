use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::stock_plate;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = stock_plate)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StockPlate {
    pub id: i32,
    pub plate_code: String,
    pub name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_plate)]
pub struct NewStockPlate {
    pub plate_code: String,
    pub name: String,
}

#[derive(AsChangeset, Debug, Default, Clone)]
#[diesel(table_name = stock_plate)]
pub struct UpdateStockPlate {
    pub plate_code: Option<String>,
    pub name: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
