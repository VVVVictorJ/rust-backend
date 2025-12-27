use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use chrono::{DateTime, FixedOffset, Utc};

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

/// 查询当天（UTC+8 时区）创建的所有不重复股票代码
pub fn get_distinct_codes_today(conn: &mut PgPoolConn) -> Result<Vec<String>, diesel::result::Error> {
    // 使用 UTC+8 时区（东八区，北京时间）
    let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_local = Utc::now().with_timezone(&utc_plus_8);
    
    // 获取当地时间的当天 00:00:00
    let today_start_local = now_local.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        today_start_local - chrono::Duration::hours(8),
        Utc
    );
    
    // 获取当地时间的明天 00:00:00
    let tomorrow_start_local = (now_local.date_naive() + chrono::Days::new(1)).and_hms_opt(0, 0, 0).unwrap();
    let tomorrow_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        tomorrow_start_local - chrono::Duration::hours(8),
        Utc
    );
    
    stock_snapshots
        .select(stock_code)
        .filter(created_at.ge(today_start_utc))
        .filter(created_at.lt(tomorrow_start_utc))
        .distinct()
        .load::<String>(conn)
}

