use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{Jsonb, Numeric, Text};
use serde_json::Value;

use crate::models::{NewStockSnapshot, StockSnapshot};
use crate::schema::stock_snapshots::dsl::{
    created_at, id, request_id, stock_code as stock_code_col, stock_snapshots,
};
use crate::utils::stock_name_filter;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn find_by_id(
    conn: &mut PgPoolConn,
    snapshot_id: i32,
) -> Result<StockSnapshot, diesel::result::Error> {
    stock_snapshots.find(snapshot_id).first(conn)
}

pub fn delete_by_id(
    conn: &mut PgPoolConn,
    snapshot_id: i32,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(stock_snapshots.find(snapshot_id)).execute(conn)
}

pub fn create(
    conn: &mut PgPoolConn,
    new_rec: &NewStockSnapshot,
) -> Result<i32, diesel::result::Error> {
    diesel::insert_into(stock_snapshots)
        .values(new_rec)
        .returning(id)
        .get_result(conn)
}

/// 根据 request_id 获取所有快照
pub fn find_by_request_id(
    conn: &mut PgPoolConn,
    req_id: i32,
) -> Result<Vec<StockSnapshot>, diesel::result::Error> {
    stock_snapshots
        .filter(request_id.eq(req_id))
        .load::<StockSnapshot>(conn)
}

/// 获取昨日（UTC+8 时区）创建的快照，根据 request_ids 过滤
#[allow(dead_code)]
pub fn find_yesterday_snapshots(
    conn: &mut PgPoolConn,
    request_ids: &[i32],
) -> Result<Vec<StockSnapshot>, diesel::result::Error> {
    if request_ids.is_empty() {
        return Ok(Vec::new());
    }

    // 使用 UTC+8 时区（东八区，北京时间）
    let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_local = Utc::now().with_timezone(&utc_plus_8);

    // 获取昨天的日期范围
    let yesterday = now_local.date_naive() - chrono::Days::new(1);
    let yesterday_start_local = yesterday.and_hms_opt(0, 0, 0).unwrap();
    let yesterday_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        yesterday_start_local - chrono::Duration::hours(8),
        Utc,
    );

    let today_start_local = now_local.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        today_start_local - chrono::Duration::hours(8),
        Utc,
    );

    stock_snapshots
        .filter(request_id.eq_any(request_ids))
        .filter(created_at.ge(yesterday_start_utc))
        .filter(created_at.lt(today_start_utc))
        .load::<StockSnapshot>(conn)
}

/// 查询当天（UTC+8 时区）创建的所有不重复股票代码
pub fn get_distinct_codes_today(
    conn: &mut PgPoolConn,
) -> Result<Vec<String>, diesel::result::Error> {
    // 使用 UTC+8 时区（东八区，北京时间）
    let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let now_local = Utc::now().with_timezone(&utc_plus_8);

    // 获取当地时间的当天 00:00:00
    let today_start_local = now_local.date_naive().and_hms_opt(0, 0, 0).unwrap();
    let today_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        today_start_local - chrono::Duration::hours(8),
        Utc,
    );

    // 获取当地时间的明天 00:00:00
    let tomorrow_start_local = (now_local.date_naive() + chrono::Days::new(1))
        .and_hms_opt(0, 0, 0)
        .unwrap();
    let tomorrow_start_utc: DateTime<Utc> = DateTime::<Utc>::from_naive_utc_and_offset(
        tomorrow_start_local - chrono::Duration::hours(8),
        Utc,
    );

    stock_snapshots
        .select(stock_code_col)
        .filter(created_at.ge(today_start_utc))
        .filter(created_at.lt(tomorrow_start_utc))
        .distinct()
        .load::<String>(conn)
}

#[derive(Debug, QueryableByName)]
pub struct StockCodeName {
    #[diesel(sql_type = Text, column_name = "stock_code")]
    pub code: String,
    #[diesel(sql_type = Text, column_name = "stock_name")]
    pub name: String,
}

/// 获取去重后的 stock_code + stock_name（按 created_at 倒序取最新）
pub fn list_distinct_codes_with_name(
    conn: &mut PgPoolConn,
) -> Result<Vec<StockCodeName>, diesel::result::Error> {
    let query = r#"
        SELECT DISTINCT ON (stock_code)
            stock_code,
            stock_name
        FROM stock_snapshots
        ORDER BY stock_code, created_at DESC
    "#;

    diesel::sql_query(query).load::<StockCodeName>(conn)
}

/// 每股取 `created_at` 最新的一条快照（代码 / 名称 / 最新价 / 板块 JSON，与交易日查询聚合方式一致）
#[derive(Debug, QueryableByName)]
pub struct LatestSnapshotFields {
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Text)]
    pub stock_name: String,
    #[diesel(sql_type = Numeric)]
    pub latest_price: BigDecimal,
    #[diesel(sql_type = Jsonb)]
    pub plates: Value,
}

/// 每只股取最新一条快照（代码 / 名称 / 价 / 板块），**不包含** ST、*ST、S*ST、SST 等特殊处理简称。
pub fn list_latest_snapshot_fields_per_stock(
    conn: &mut PgPoolConn,
) -> Result<Vec<LatestSnapshotFields>, diesel::result::Error> {
    let query = r#"
        WITH latest_snap AS (
            SELECT DISTINCT ON (stock_code)
                stock_code,
                stock_name,
                latest_price
            FROM stock_snapshots
            ORDER BY stock_code, created_at DESC
        )
        SELECT
            ls.stock_code,
            ls.stock_name,
            ls.latest_price,
            COALESCE(
                jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
                    FILTER (WHERE sp.id IS NOT NULL),
                '[]'::jsonb
            ) AS plates
        FROM latest_snap ls
        LEFT JOIN stock_table st ON ls.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        GROUP BY ls.stock_code, ls.stock_name, ls.latest_price
        ORDER BY ls.stock_code
    "#;

    let mut rows = diesel::sql_query(query).load::<LatestSnapshotFields>(conn)?;
    rows.retain(|r| !stock_name_filter::is_st_special_stock_name(&r.stock_name));
    Ok(rows)
}
