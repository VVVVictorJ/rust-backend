use chrono::NaiveDate;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Jsonb, Text};
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

#[derive(Debug, QueryableByName)]
pub struct PlateStatisticsResult {
    #[diesel(sql_type = Text)]
    pub plate_code: String,
    #[diesel(sql_type = Text)]
    pub plate_name: String,
    #[diesel(sql_type = BigInt)]
    pub stock_count: i64,
    #[diesel(sql_type = Jsonb)]
    pub stocks: Value,
}

#[derive(Debug, QueryableByName)]
pub struct PlateStatisticsSummary {
    #[diesel(sql_type = BigInt)]
    pub total_stock_count: i64,
    #[diesel(sql_type = BigInt)]
    pub classified_stock_count: i64,
}

pub fn query_plate_statistics(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
) -> Result<Vec<PlateStatisticsResult>, diesel::result::Error> {
    let query = r#"
        WITH day_plate_stocks AS (
            SELECT DISTINCT
                sp.plate_code,
                sp.name AS plate_name,
                a.stock_code,
                a.stock_name
            FROM stock_snapshots a
            INNER JOIN stock_table st ON a.stock_code = st.stock_code
            INNER JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
            INNER JOIN stock_plate sp ON sps.plate_id = sp.id
            WHERE (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date
        )
        SELECT
            plate_code,
            plate_name,
            COUNT(*) AS stock_count,
            jsonb_agg(
                jsonb_build_object('stock_code', stock_code, 'stock_name', stock_name)
                ORDER BY stock_code
            ) AS stocks
        FROM day_plate_stocks
        GROUP BY plate_code, plate_name
        ORDER BY stock_count DESC, plate_name ASC;
    "#;

    diesel::sql_query(query)
        .bind::<diesel::sql_types::Date, _>(trade_date)
        .load::<PlateStatisticsResult>(conn)
}

pub fn query_plate_statistics_summary(
    conn: &mut PgPoolConn,
    trade_date: NaiveDate,
) -> Result<PlateStatisticsSummary, diesel::result::Error> {
    let query = r#"
        WITH day_stocks AS (
            SELECT DISTINCT
                a.stock_code
            FROM stock_snapshots a
            WHERE (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = $1::date
        ),
        classified_stocks AS (
            SELECT DISTINCT
                ds.stock_code
            FROM day_stocks ds
            INNER JOIN stock_table st ON ds.stock_code = st.stock_code
            INNER JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        )
        SELECT
            (SELECT COUNT(*) FROM day_stocks) AS total_stock_count,
            (SELECT COUNT(*) FROM classified_stocks) AS classified_stock_count;
    "#;

    diesel::sql_query(query)
        .bind::<diesel::sql_types::Date, _>(trade_date)
        .get_result::<PlateStatisticsSummary>(conn)
}
