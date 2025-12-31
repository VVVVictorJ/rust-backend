use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Date, Numeric, Text, Timestamptz};
use bigdecimal::BigDecimal;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 价格对比查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct PriceCompareResult {
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Text)]
    pub stock_name: String,
    #[diesel(sql_type = Numeric)]
    pub latest_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub high_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub close_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub open_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub low_price: BigDecimal,
    #[diesel(sql_type = Text)]
    pub grade: String,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
}

/// 根据快照日期和交易日期查询价格对比数据（分页）
pub fn query_price_compare(
    conn: &mut PgPoolConn,
    snapshot_date: NaiveDate,
    trade_date: NaiveDate,
    limit: i64,
    offset: i64,
) -> Result<Vec<PriceCompareResult>, diesel::result::Error> {
    let query = r#"
        SELECT DISTINCT 
            b.stock_code,
            b.stock_name,
            b.latest_price,
            c.high_price,
            c.close_price,
            c.open_price,
            c.low_price,
            CASE 
                WHEN a.profit_rate = 2 THEN 'A'
                WHEN a.profit_rate = 1 THEN 'B'
                ELSE 'C'
            END as grade,
            b.created_at 
        FROM profit_analysis a 
        LEFT JOIN stock_snapshots b ON a.snapshot_id = b.id 
        LEFT JOIN daily_klines c ON b.stock_code = c.stock_code 
        WHERE a.profit_rate IN (1, 2) 
          AND b.created_at::date = $1
          AND c.trade_date = $2
        ORDER BY b.stock_code
        LIMIT $3 OFFSET $4
    "#;

    diesel::sql_query(query)
        .bind::<Date, _>(snapshot_date)
        .bind::<Date, _>(trade_date)
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<PriceCompareResult>(conn)
}

/// 查询总记录数
pub fn count_price_compare(
    conn: &mut PgPoolConn,
    snapshot_date: NaiveDate,
    trade_date: NaiveDate,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let query = r#"
        SELECT COUNT(*) as count
        FROM (
            SELECT DISTINCT 
                b.stock_code,
                b.stock_name,
                b.latest_price,
                c.high_price,
                c.close_price,
                c.open_price,
                c.low_price,
                CASE 
                    WHEN a.profit_rate = 2 THEN 'A'
                    WHEN a.profit_rate = 1 THEN 'B'
                    ELSE 'C'
                END as grade,
                b.created_at 
            FROM profit_analysis a 
            LEFT JOIN stock_snapshots b ON a.snapshot_id = b.id 
            LEFT JOIN daily_klines c ON b.stock_code = c.stock_code 
            WHERE a.profit_rate IN (1, 2) 
              AND b.created_at::date = $1
              AND c.trade_date = $2
        ) AS distinct_results
    "#;

    let result = diesel::sql_query(query)
        .bind::<Date, _>(snapshot_date)
        .bind::<Date, _>(trade_date)
        .get_result::<CountResult>(conn)?;

    Ok(result.count)
}

