use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Numeric, Text, Timestamptz, Nullable, Jsonb};
use bigdecimal::BigDecimal;
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 查询结果结构体，用于接收 SQL 查询的结果
#[derive(Debug, QueryableByName)]
pub struct TradeDateQueryResult {
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Text)]
    pub stock_name: String,
    #[diesel(sql_type = Numeric)]
    pub latest_price: BigDecimal,
    #[diesel(sql_type = Nullable<Numeric>)]
    pub close_price: Option<BigDecimal>,
    #[diesel(sql_type = Numeric)]
    pub change_pct: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub volume_ratio: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub turnover_rate: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub bid_ask_ratio: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub main_force_inflow: BigDecimal,
    #[diesel(sql_type = Timestamptz)]
    pub created_at: DateTime<Utc>,
    #[diesel(sql_type = Jsonb)]
    pub plates: Value,
}

/// 根据交易日期查询股票快照数据（分页）
pub fn query_by_trade_date(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
    limit: i64,
    offset: i64,
) -> Result<Vec<TradeDateQueryResult>, diesel::result::Error> {
    let query = r#"
        SELECT 
            a.stock_code,
            a.stock_name,
            a.latest_price,
            dk.close_price,
            a.change_pct,
            a.volume_ratio,
            a.turnover_rate,
            a.bid_ask_ratio,
            a.main_force_inflow,
            a.created_at,
            COALESCE(
                jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
                    FILTER (WHERE sp.id IS NOT NULL),
                '[]'::jsonb
            ) AS plates                     
        FROM stock_snapshots a 
        LEFT JOIN daily_klines dk 
            ON a.stock_code = dk.stock_code 
            AND dk.trade_date = $1  
        LEFT JOIN stock_table st ON a.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        WHERE 
            (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date 
        GROUP BY
            a.stock_code,
            a.stock_name,
            a.latest_price,
            dk.close_price,
            a.change_pct,
            a.volume_ratio,
            a.turnover_rate,
            a.bid_ask_ratio,
            a.main_force_inflow,
            a.created_at
        ORDER BY a.main_force_inflow DESC
        LIMIT $2 OFFSET $3;
    "#;
    // let query = r#"
    //     SELECT 
    //         a.stock_code,
    //         a.stock_name,
    //         a.latest_price,
    //         dk.close_price,
    //         a.change_pct,
    //         a.volume_ratio,
    //         a.turnover_rate,
    //         a.bid_ask_ratio,
    //         a.main_force_inflow,
    //         a.created_at,
    //         COALESCE(
    //             jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
    //                 FILTER (WHERE sp.id IS NOT NULL),
    //             '[]'::jsonb
    //         ) AS plates
    //     FROM stock_snapshots a 
    //     LEFT JOIN daily_klines dk ON a.stock_code = dk.stock_code 
    //     LEFT JOIN stock_table st ON a.stock_code = st.stock_code
    //     LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
    //     LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
    //     WHERE a.main_force_inflow > 0
    //       AND (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = dk.trade_date 
    //       AND dk.trade_date = $1
    //     GROUP BY
    //         a.stock_code,
    //         a.stock_name,
    //         a.latest_price,
    //         dk.close_price,
    //         a.change_pct,
    //         a.volume_ratio,
    //         a.turnover_rate,
    //         a.bid_ask_ratio,
    //         a.main_force_inflow,
    //         a.created_at
    //     ORDER BY a.main_force_inflow DESC
    //     LIMIT $2 OFFSET $3
    // "#;

    diesel::sql_query(query)
        .bind::<diesel::sql_types::Date, _>(trade_date)
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<TradeDateQueryResult>(conn)
}

/// 查询总记录数
pub fn count_by_trade_date(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let query = r#"
    SELECT COUNT(*) AS count
    FROM stock_snapshots a 
    WHERE 
    (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date;
    "#;

    let result = diesel::sql_query(query)
        .bind::<diesel::sql_types::Date, _>(trade_date)
        .get_result::<CountResult>(conn)?;

    Ok(result.count)
}

