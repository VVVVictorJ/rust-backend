use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Jsonb, Nullable, Numeric, Text, Timestamptz};
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 查询结果结构体，用于接收 SQL 查询的结果
#[derive(Debug, QueryableByName)]
pub struct StockAppearanceQueryResult {
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

/// 按条件查询股票快照历史出现记录（分页）
/// stock_code / stock_name 模糊匹配，plate_code 精确匹配（按板块成分股过滤）
pub fn query_by_condition(
    conn: &mut PgPoolConn,
    stock_code: Option<String>,
    stock_name: Option<String>,
    plate_code: Option<String>,
    limit: i64,
    offset: i64,
) -> Result<Vec<StockAppearanceQueryResult>, diesel::result::Error> {
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
            ($1::text IS NULL OR a.stock_code ILIKE '%' || $1 || '%')
            AND ($2::text IS NULL OR a.stock_name ILIKE '%' || $2 || '%')
            AND (
                $3::text IS NULL OR EXISTS (
                    SELECT 1
                    FROM stock_table st2
                    JOIN stock_plate_stock_table sps2 ON st2.id = sps2.stock_table_id
                    JOIN stock_plate sp2 ON sps2.plate_id = sp2.id
                    WHERE st2.stock_code = a.stock_code
                      AND sp2.plate_code = $3
                )
            )
        GROUP BY
            a.id,
            dk.close_price
        ORDER BY a.created_at DESC
        LIMIT $4 OFFSET $5;
    "#;

    diesel::sql_query(query)
        .bind::<Nullable<Text>, _>(stock_code)
        .bind::<Nullable<Text>, _>(stock_name)
        .bind::<Nullable<Text>, _>(plate_code)
        .bind::<BigInt, _>(limit)
        .bind::<BigInt, _>(offset)
        .load::<StockAppearanceQueryResult>(conn)
}

/// 按条件查询总记录数
pub fn count_by_condition(
    conn: &mut PgPoolConn,
    stock_code: Option<String>,
    stock_name: Option<String>,
    plate_code: Option<String>,
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
            ($1::text IS NULL OR a.stock_code ILIKE '%' || $1 || '%')
            AND ($2::text IS NULL OR a.stock_name ILIKE '%' || $2 || '%')
            AND (
                $3::text IS NULL OR EXISTS (
                    SELECT 1
                    FROM stock_table st2
                    JOIN stock_plate_stock_table sps2 ON st2.id = sps2.stock_table_id
                    JOIN stock_plate sp2 ON sps2.plate_id = sp2.id
                    WHERE st2.stock_code = a.stock_code
                      AND sp2.plate_code = $3
                )
            );
    "#;

    let result = diesel::sql_query(query)
        .bind::<Nullable<Text>, _>(stock_code)
        .bind::<Nullable<Text>, _>(stock_name)
        .bind::<Nullable<Text>, _>(plate_code)
        .get_result::<CountResult>(conn)?;

    Ok(result.count)
}
