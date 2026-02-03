use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Date, Numeric, Text, Timestamptz, Nullable, Jsonb};
use bigdecimal::BigDecimal;
use serde_json::Value;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

/// 观察表查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct WatchlistQueryResult {
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Nullable<Text>)]
    pub stock_name: Option<String>,
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

/// 观察表明细查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct WatchlistDetailResult {
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

/// 观察表K线查询结果结构体
#[derive(Debug, QueryableByName)]
pub struct WatchlistKlineResult {
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Date)]
    pub trade_date: NaiveDate,
    #[diesel(sql_type = Numeric)]
    pub open_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub high_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub low_price: BigDecimal,
    #[diesel(sql_type = Numeric)]
    pub close_price: BigDecimal,
    #[diesel(sql_type = BigInt)]
    pub volume: i64,
    #[diesel(sql_type = Numeric)]
    pub amount: BigDecimal,
}

/// 查询观察表中的股票，支持板块和区间筛选
#[allow(clippy::too_many_arguments)]
pub fn query_watchlist_stocks(
    conn: &mut PgPoolConn,
    plate_codes: &[String],
    change_pct_min: Option<&BigDecimal>,
    change_pct_max: Option<&BigDecimal>,
    volume_ratio_min: Option<&BigDecimal>,
    volume_ratio_max: Option<&BigDecimal>,
    turnover_rate_min: Option<&BigDecimal>,
    turnover_rate_max: Option<&BigDecimal>,
    bid_ask_ratio_min: Option<&BigDecimal>,
    bid_ask_ratio_max: Option<&BigDecimal>,
    main_force_inflow_min: Option<&BigDecimal>,
    main_force_inflow_max: Option<&BigDecimal>,
    stock_code_filter: Option<&str>,
) -> Result<Vec<WatchlistQueryResult>, diesel::result::Error> {
    // 构建 WHERE 条件
    let mut where_conditions = Vec::new();

    // 板块筛选
    if !plate_codes.is_empty() {
        let plate_list: Vec<String> = plate_codes.iter().map(|c| format!("'{}'", c.replace("'", "''"))).collect();
        where_conditions.push(format!("sp.plate_code IN ({})", plate_list.join(",")));
    }

    // 区间筛选
    if let Some(v) = change_pct_min {
        where_conditions.push(format!("ls.change_pct >= {v}"));
    }
    if let Some(v) = change_pct_max {
        where_conditions.push(format!("ls.change_pct <= {v}"));
    }
    if let Some(v) = volume_ratio_min {
        where_conditions.push(format!("ls.volume_ratio >= {v}"));
    }
    if let Some(v) = volume_ratio_max {
        where_conditions.push(format!("ls.volume_ratio <= {v}"));
    }
    if let Some(v) = turnover_rate_min {
        where_conditions.push(format!("ls.turnover_rate >= {v}"));
    }
    if let Some(v) = turnover_rate_max {
        where_conditions.push(format!("ls.turnover_rate <= {v}"));
    }
    if let Some(v) = bid_ask_ratio_min {
        where_conditions.push(format!("ls.bid_ask_ratio >= {v}"));
    }
    if let Some(v) = bid_ask_ratio_max {
        where_conditions.push(format!("ls.bid_ask_ratio <= {v}"));
    }
    if let Some(v) = main_force_inflow_min {
        where_conditions.push(format!("ls.main_force_inflow >= {v}"));
    }
    if let Some(v) = main_force_inflow_max {
        where_conditions.push(format!("ls.main_force_inflow <= {v}"));
    }

    // 股票代码模糊匹配
    if let Some(filter) = stock_code_filter {
        let escaped = filter.replace("'", "''").replace("%", "\\%").replace("_", "\\_");
        where_conditions.push(format!("ls.stock_code LIKE '%{escaped}%'"));
    }

    let where_clause = if where_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", where_conditions.join(" AND "))
    };

    let query = format!(
        r#"
        WITH latest_snapshots AS (
            SELECT DISTINCT ON (sw.stock_code)
                sw.stock_code,
                sw.stock_name,
                ss.latest_price,
                dk.close_price,
                ss.change_pct,
                ss.volume_ratio,
                ss.turnover_rate,
                ss.bid_ask_ratio,
                ss.main_force_inflow,
                ss.created_at
            FROM stock_watchlist sw
            LEFT JOIN stock_snapshots ss ON sw.stock_code = ss.stock_code
            LEFT JOIN daily_klines dk ON ss.stock_code = dk.stock_code 
                AND dk.trade_date = (ss.created_at AT TIME ZONE 'Asia/Shanghai')::date
            WHERE ss.id IS NOT NULL
            ORDER BY sw.stock_code, ss.created_at DESC
        )
        SELECT 
            ls.stock_code,
            ls.stock_name,
            ls.latest_price,
            ls.close_price,
            ls.change_pct,
            ls.volume_ratio,
            ls.turnover_rate,
            ls.bid_ask_ratio,
            ls.main_force_inflow,
            ls.created_at,
            COALESCE(
                jsonb_agg(DISTINCT jsonb_build_object('plate_code', sp.plate_code, 'name', sp.name))
                    FILTER (WHERE sp.id IS NOT NULL),
                '[]'::jsonb
            ) AS plates
        FROM latest_snapshots ls
        LEFT JOIN stock_table st ON ls.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        {where_clause}
        GROUP BY
            ls.stock_code,
            ls.stock_name,
            ls.latest_price,
            ls.close_price,
            ls.change_pct,
            ls.volume_ratio,
            ls.turnover_rate,
            ls.bid_ask_ratio,
            ls.main_force_inflow,
            ls.created_at
        ORDER BY ls.main_force_inflow DESC;
        "#
    );

    diesel::sql_query(query).load::<WatchlistQueryResult>(conn)
}

/// 查询股票的时间序列明细（从 stock_snapshots）
pub fn query_stock_snapshot_detail(
    conn: &mut PgPoolConn,
    stock_code: &str,
) -> Result<Vec<WatchlistDetailResult>, diesel::result::Error> {
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
        LEFT JOIN daily_klines dk ON a.stock_code = dk.stock_code 
            AND dk.trade_date = (a.created_at AT TIME ZONE 'Asia/Shanghai')::date
        LEFT JOIN stock_table st ON a.stock_code = st.stock_code
        LEFT JOIN stock_plate_stock_table sps ON st.id = sps.stock_table_id
        LEFT JOIN stock_plate sp ON sps.plate_id = sp.id
        WHERE a.stock_code = $1
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
        .load::<WatchlistDetailResult>(conn)
}

/// 查找股票在 stock_snapshots 中首次出现的日期
pub fn find_first_occurrence_date(
    conn: &mut PgPoolConn,
    stock_code: &str,
) -> Result<Option<NaiveDate>, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct DateResult {
        #[diesel(sql_type = Date)]
        first_date: NaiveDate,
    }

    let query = r#"
        SELECT MIN((created_at AT TIME ZONE 'Asia/Shanghai')::date) AS first_date
        FROM stock_snapshots
        WHERE stock_code = $1;
    "#;

    diesel::sql_query(query)
        .bind::<Text, _>(stock_code)
        .get_result::<DateResult>(conn)
        .optional()
        .map(|opt| opt.map(|r| r.first_date))
}

/// 查询股票的 K 线数据（按日期范围）
pub fn query_stock_kline_range(
    conn: &mut PgPoolConn,
    stock_code: &str,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<WatchlistKlineResult>, diesel::result::Error> {
    let query = r#"
        SELECT 
            stock_code,
            trade_date,
            open_price,
            high_price,
            low_price,
            close_price,
            volume,
            amount
        FROM daily_klines
        WHERE stock_code = $1
          AND trade_date >= $2
          AND trade_date <= $3
        ORDER BY trade_date ASC;
    "#;

    diesel::sql_query(query)
        .bind::<Text, _>(stock_code)
        .bind::<Date, _>(start_date)
        .bind::<Date, _>(end_date)
        .load::<WatchlistKlineResult>(conn)
}
