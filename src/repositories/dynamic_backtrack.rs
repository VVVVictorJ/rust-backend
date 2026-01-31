use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Date, Integer, Numeric, Text, Timestamptz, Nullable, Jsonb};
use bigdecimal::BigDecimal;
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 动态回溯查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct DynamicBacktrackResult {
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
    #[diesel(sql_type = Integer)]
    pub occurrence_count: i32,
    #[diesel(sql_type = Jsonb)]
    pub plates: Value,
}

/// 查询动态回溯股票列表
/// 基于 stock_trading_calendar 表获取真正的A股交易日
pub fn query_dynamic_backtrack(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
    trade_days: i32,
    min_occurrences: i32,
) -> Result<Vec<DynamicBacktrackResult>, diesel::result::Error> {
    let query = r#"
        WITH trading_days AS (
            -- 从 stock_trading_calendar 获取 N 个交易日（包括当天）
            SELECT trade_date
            FROM stock_trading_calendar
            WHERE trade_date <= $1::date
              AND is_holiday = FALSE
            ORDER BY trade_date DESC
            LIMIT $2
        ),
        target_date_stocks AS (
            -- 查询指定日期的股票快照（去重取每只股票最新的一条）
            SELECT DISTINCT ON (stock_code)
                stock_code,
                stock_name,
                latest_price,
                change_pct,
                volume_ratio,
                turnover_rate,
                bid_ask_ratio,
                main_force_inflow,
                created_at
            FROM stock_snapshots
            WHERE (created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date
            ORDER BY stock_code, created_at DESC
        ),
        occurrence_counts AS (
            -- 统计每只股票在这些交易日的出现天数
            SELECT 
                tds.stock_code,
                COUNT(DISTINCT (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date)::integer AS occurrence_count
            FROM target_date_stocks tds
            LEFT JOIN stock_snapshots ss ON tds.stock_code = ss.stock_code
                AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date IN (SELECT trade_date FROM trading_days)
            GROUP BY tds.stock_code
            HAVING COUNT(DISTINCT (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date) >= $3
        )
        SELECT 
            tds.stock_code,
            tds.stock_name,
            tds.latest_price,
            dk.close_price,
            tds.change_pct,
            tds.volume_ratio,
            tds.turnover_rate,
            tds.bid_ask_ratio,
            tds.main_force_inflow,
            tds.created_at,
            oc.occurrence_count,
            COALESCE(
                jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
                    FILTER (WHERE sp.id IS NOT NULL),
                '[]'::jsonb
            ) AS plates
        FROM target_date_stocks tds
        INNER JOIN occurrence_counts oc ON tds.stock_code = oc.stock_code
        LEFT JOIN daily_klines dk ON tds.stock_code = dk.stock_code AND dk.trade_date = $1
        LEFT JOIN stock_table st ON tds.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        GROUP BY 
            tds.stock_code,
            tds.stock_name,
            tds.latest_price,
            dk.close_price,
            tds.change_pct,
            tds.volume_ratio,
            tds.turnover_rate,
            tds.bid_ask_ratio,
            tds.main_force_inflow,
            tds.created_at,
            oc.occurrence_count
        ORDER BY oc.occurrence_count DESC,
                 tds.main_force_inflow DESC;
    "#;

    diesel::sql_query(query)
        .bind::<Date, _>(trade_date)
        .bind::<Integer, _>(trade_days)
        .bind::<Integer, _>(min_occurrences)
        .load::<DynamicBacktrackResult>(conn)
}
