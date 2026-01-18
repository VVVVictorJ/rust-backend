use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::stock_table;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = stock_table)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StockTable {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_table)]
pub struct NewStockTable {
    pub stock_code: String,
    pub stock_name: String,
}

#[derive(AsChangeset, Debug, Default, Clone)]
#[diesel(table_name = stock_table)]
pub struct UpdateStockTable {
    pub stock_code: Option<String>,
    pub stock_name: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
