use diesel::prelude::*;
use crate::schema::stock_request_stocks;

#[derive(Queryable, Debug, Clone)]
#[diesel(belongs_to(crate::models::stock_requests::StockRequest, foreign_key = request_id))]
pub struct StockRequestStock {
    pub request_id: i32,
    pub stock_code: String,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = stock_request_stocks)]
pub struct NewStockRequestStock {
    pub request_id: i32,
    pub stock_code: String,
}

