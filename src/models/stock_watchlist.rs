use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::schema::stock_watchlist;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = stock_watchlist)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct StockWatchlist {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_watchlist)]
pub struct NewStockWatchlist {
    pub stock_code: String,
    pub stock_name: Option<String>,
}

#[derive(AsChangeset, Debug, Default, Clone)]
#[diesel(table_name = stock_watchlist)]
pub struct UpdateStockWatchlist {
    pub stock_name: Option<String>,
    pub updated_at: Option<NaiveDateTime>,
}
