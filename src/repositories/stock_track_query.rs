use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Date, Integer, Numeric, Text, Timestamptz, Nullable, Jsonb};
use bigdecimal::BigDecimal;
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 交易日股票快照结果（含出现次数统计）
#[derive(Debug, QueryableByName)]
pub struct TrackQueryResult {
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
    pub days_3_count: i32,
    #[diesel(sql_type = Integer)]
    pub days_7_count: i32,
    #[diesel(sql_type = Integer)]
    pub days_14_count: i32,
    #[diesel(sql_type = Jsonb)]
    pub plates: Value,
}

/// 追踪明细查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct TrackDetailResult {
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

/// 查询指定交易日的股票快照，并统计每只股票在过去3/7/14天的出现次数
/// 只返回在任一周期内出现次数 >= min_occurrences 的股票
pub fn query_tracked_stocks_by_date(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
    min_occurrences: i32,
) -> Result<Vec<TrackQueryResult>, diesel::result::Error> {
    let query = r#"
        WITH target_date_stocks AS (
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
            -- 统计每只股票在过去N天的出现天数
            SELECT 
                tds.stock_code,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '3 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_3_count,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '7 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_7_count,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '14 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_14_count
            FROM target_date_stocks tds
            LEFT JOIN stock_snapshots ss ON tds.stock_code = ss.stock_code
                AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '14 days'
                AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
            GROUP BY tds.stock_code
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
            COALESCE(oc.days_3_count, 0) AS days_3_count,
            COALESCE(oc.days_7_count, 0) AS days_7_count,
            COALESCE(oc.days_14_count, 0) AS days_14_count,
            COALESCE(
                jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
                    FILTER (WHERE sp.id IS NOT NULL),
                '[]'::jsonb
            ) AS plates
        FROM target_date_stocks tds
        LEFT JOIN occurrence_counts oc ON tds.stock_code = oc.stock_code
        LEFT JOIN daily_klines dk ON tds.stock_code = dk.stock_code AND dk.trade_date = $1
        LEFT JOIN stock_table st ON tds.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        WHERE COALESCE(oc.days_3_count, 0) >= 2
           OR COALESCE(oc.days_7_count, 0) >= 2
           OR COALESCE(oc.days_14_count, 0) >= $2
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
            oc.days_3_count,
            oc.days_7_count,
            oc.days_14_count
        ORDER BY GREATEST(COALESCE(oc.days_14_count, 0), COALESCE(oc.days_7_count, 0), COALESCE(oc.days_3_count, 0)) DESC,
                 tds.main_force_inflow DESC;
    "#;

    diesel::sql_query(query)
        .bind::<Date, _>(trade_date)
        .bind::<Integer, _>(min_occurrences)
        .load::<TrackQueryResult>(conn)
}

/// 统计满足条件的股票总数
#[allow(dead_code)]
pub fn count_tracked_stocks_by_date(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
    min_occurrences: i32,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let query = r#"
        WITH target_date_stocks AS (
            SELECT DISTINCT stock_code
            FROM stock_snapshots
            WHERE (created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date
        ),
        occurrence_counts AS (
            SELECT 
                tds.stock_code,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '3 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_3_count,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '7 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_7_count,
                COUNT(DISTINCT CASE 
                    WHEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '14 days'
                         AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
                    THEN (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date 
                END)::integer AS days_14_count
            FROM target_date_stocks tds
            LEFT JOIN stock_snapshots ss ON tds.stock_code = ss.stock_code
                AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $1::date - INTERVAL '14 days'
                AND (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date < $1::date
            GROUP BY tds.stock_code
        )
        SELECT COUNT(*) AS count
        FROM occurrence_counts
        WHERE days_3_count >= 2 OR days_7_count >= 2 OR days_14_count >= $2;
    "#;

    let result = diesel::sql_query(query)
        .bind::<Date, _>(trade_date)
        .bind::<Integer, _>(min_occurrences)
        .get_result::<CountResult>(conn)?;

    Ok(result.count)
}

/// 查询某只股票在指定日期之前N天内的时间序列明细
pub fn query_stock_track_detail(
    conn: &mut PgPoolConn,
    stock_code: &str,
    trade_date: NaiveDate,
    track_days: i32,
) -> Result<Vec<TrackDetailResult>, diesel::result::Error> {
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
            AND dk.trade_date = (a.created_at AT TIME ZONE 'Asia/Shanghai')::date
        LEFT JOIN stock_table st ON a.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        WHERE 
            a.stock_code = $1
            AND (a.created_at AT TIME ZONE 'Asia/Shanghai')::date >= $2::date - ($3 || ' days')::interval
            AND (a.created_at AT TIME ZONE 'Asia/Shanghai')::date <= $2::date
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
        ORDER BY a.created_at DESC;
    "#;

    diesel::sql_query(query)
        .bind::<Text, _>(stock_code)
        .bind::<Date, _>(trade_date)
        .bind::<Integer, _>(track_days)
        .load::<TrackDetailResult>(conn)
}

/// 统计某只股票在指定日期之前N天内的出现次数
#[allow(dead_code)]
pub fn count_stock_track_detail(
    conn: &mut PgPoolConn,
    stock_code: &str,
    trade_date: NaiveDate,
    track_days: i32,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    let query = r#"
        SELECT COUNT(*) AS count
        FROM stock_snapshots
        WHERE 
            stock_code = $1
            AND (created_at AT TIME ZONE 'Asia/Shanghai')::date >= $2::date - ($3 || ' days')::interval
            AND (created_at AT TIME ZONE 'Asia/Shanghai')::date <= $2::date;
    "#;

    let result = diesel::sql_query(query)
        .bind::<Text, _>(stock_code)
        .bind::<Date, _>(trade_date)
        .bind::<Integer, _>(track_days)
        .get_result::<CountResult>(conn)?;

    Ok(result.count)
}
