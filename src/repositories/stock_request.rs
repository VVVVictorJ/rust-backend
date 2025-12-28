use chrono::NaiveDate;
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

/// 查找所有 time_range_end 为空的请求（待处理）
pub fn find_pending_requests(conn: &mut PgPoolConn) -> Result<Vec<StockRequest>, diesel::result::Error> {
    stock_requests
        .filter(time_range_end.is_null())
        .load::<StockRequest>(conn)
}

/// 更新 time_range_end 字段，标记处理完成
pub fn update_time_range_end(conn: &mut PgPoolConn, req_id: i32, end_date: NaiveDate) -> Result<usize, diesel::result::Error> {
    diesel::update(stock_requests.find(req_id))
        .set(time_range_end.eq(Some(end_date)))
        .execute(conn)
}

