use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{FixedOffset, NaiveDate, Utc};
use serde_json::json;

use crate::api_models::ai_analysis::{
    TrendDetailResponse, TrendHistoryItem, TrendHistoryRequest, TrendHistoryResponse,
    TrendPredictionRequest, TrendPredictionResponse,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::{NewAiTrendAnalysis, UpdateAiTrendAnalysis};
use crate::repositories::{ai_trend_analysis, daily_kline};
use crate::services::{ai_service, kline_service};
use crate::utils::http_client;

/// POST /api/ai-analysis/trend-prediction
/// 对指定股票发起趋势分析
pub async fn trend_prediction(
    State(state): State<AppState>,
    Json(payload): Json<TrendPredictionRequest>,
) -> Result<Json<TrendPredictionResponse>, AppError> {
    let stock_code_input = payload.stock_code.trim().to_string();
    if stock_code_input.is_empty() {
        return Err(AppError::BadRequest("stock_code is required".to_string()));
    }

    let start_time = std::time::Instant::now();

    // 1. 查询 stock_snapshots 获取该股票的信号数据
    let (signals, stock_name) = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        query_stock_signals(&mut conn, &stock_code_input)?
    };

    if signals.is_empty() {
        return Err(AppError::BadRequest(format!(
            "股票 {stock_code_input} 没有信号数据（stock_snapshots 中无记录）"
        )));
    }

    // 2. 找到最早信号日期
    let earliest_signal_date = signals.iter().map(|s| s.signal_date).min().unwrap();

    // 3. 查询 stock_trading_calendar，从最早信号日往前找20个交易日
    let kline_start_date = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        find_trading_date_before(&mut conn, earliest_signal_date, 20)?
    };

    // 4. 结束日期为当前日期（UTC+8）
    let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let today = Utc::now().with_timezone(&utc_plus_8).date_naive();

    // 5. 查询 daily_klines 获取K线数据
    let klines = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        query_kline_range(&mut conn, &stock_code_input, kline_start_date, today)?
    };

    // 6. 检查K线数据是否完整，不完整则自动补齐
    let klines = if klines.is_empty() || needs_kline_fill(&klines, kline_start_date, today) {
        tracing::info!(
            "K线数据不完整，自动补齐 {} 从 {} 到 {}",
            stock_code_input,
            kline_start_date,
            today
        );
        fill_missing_klines(&state, &stock_code_input, kline_start_date, today).await?;
        // 重新查询完整数据
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        query_kline_range(&mut conn, &stock_code_input, kline_start_date, today)?
    } else {
        klines
    };

    // 7. 组装 send_to_ai.json 格式的 payload
    let kline_strings: Vec<String> = klines
        .iter()
        .map(|k| {
            format!(
                "{},{},{},{},{},{}",
                k.trade_date, k.open_price, k.close_price, k.high_price, k.low_price, k.volume
            )
        })
        .collect();

    let signal_json: Vec<serde_json::Value> = signals
        .iter()
        .map(|s| {
            json!({
                "datetime": s.datetime,
                "change_pct": s.change_pct,
                "volume_ratio": s.volume_ratio,
                "turnover_rate": s.turnover_rate,
                "bid_ask_ratio": s.bid_ask_ratio,
            })
        })
        .collect();

    let ai_payload = json!({
        "stock_name": format!("{} {}", stock_code_input, stock_name.as_deref().unwrap_or("")),
        "klines": kline_strings,
        "signals": signal_json,
    });

    // 8. 先创建一条 pending 状态的记录
    let new_record = NewAiTrendAnalysis {
        stock_code: stock_code_input.clone(),
        stock_name: stock_name.clone(),
        model_name: std::env::var("QWEN_MODEL")
            .unwrap_or_else(|_| "qwen3-max-2026-01-23".to_string()),
        status: "processing".to_string(),
        request_payload: ai_payload.clone(),
        response_json: None,
        raw_response: None,
        signal_count: Some(signals.len() as i32),
        kline_start_date: Some(kline_start_date),
        kline_end_date: Some(today),
        error_message: None,
        duration_ms: None,
    };

    let record = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        ai_trend_analysis::create(&mut conn, &new_record).map_err(|e| {
            tracing::error!("Failed to create ai_trend_analysis record: {}", e);
            AppError::InternalServerError
        })?
    };

    // 9. 调用 Qwen API
    let ai_client = ai_service::create_ai_client().map_err(|e| {
        tracing::error!("Failed to create AI client: {}", e);
        AppError::InternalServerError
    })?;

    let ai_result = ai_service::call_qwen_analysis(&ai_client, &ai_payload).await;

    let duration_ms = start_time.elapsed().as_millis() as i64;

    // 10. 更新记录
    let (final_status, response_json, raw_response, error_message) = match ai_result {
        Ok(result) => (
            "completed".to_string(),
            result.response_json.clone(),
            Some(result.raw_response),
            None,
        ),
        Err(e) => {
            tracing::error!("AI analysis failed for {}: {}", stock_code_input, e);
            ("failed".to_string(), None, None, Some(e.to_string()))
        }
    };

    let update_data = UpdateAiTrendAnalysis {
        status: Some(final_status.clone()),
        response_json: response_json.clone(),
        raw_response: raw_response.clone(),
        error_message: error_message.clone(),
        duration_ms: Some(duration_ms),
        updated_at: Some(Utc::now()),
    };

    let updated_record = {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        ai_trend_analysis::update_by_id(&mut conn, record.id, &update_data).map_err(|e| {
            tracing::error!("Failed to update ai_trend_analysis record: {}", e);
            AppError::InternalServerError
        })?
    };

    if final_status == "failed" {
        return Err(AppError::BadRequest(
            error_message.unwrap_or_else(|| "AI分析失败".to_string()),
        ));
    }

    Ok(Json(to_prediction_response(&updated_record)))
}

/// GET /api/ai-analysis/trend-prediction/history
/// 查询历史分析记录
pub async fn trend_history(
    State(state): State<AppState>,
    Query(params): Query<TrendHistoryRequest>,
) -> Result<Json<TrendHistoryResponse>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let (records, total) = ai_trend_analysis::list_history(
        &mut conn,
        params.stock_code.as_deref(),
        params.page_size,
        params.page,
    )
    .map_err(|e| {
        tracing::error!("Failed to query ai analysis history: {}", e);
        AppError::InternalServerError
    })?;

    let data = records
        .into_iter()
        .map(|r| TrendHistoryItem {
            id: r.id,
            stock_code: r.stock_code,
            stock_name: r.stock_name,
            model_name: r.model_name,
            status: r.status,
            signal_count: r.signal_count,
            kline_start_date: r.kline_start_date,
            kline_end_date: r.kline_end_date,
            duration_ms: r.duration_ms,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(TrendHistoryResponse { data, total }))
}

/// GET /api/ai-analysis/trend-prediction/:id
/// 查询单条分析详情
pub async fn trend_detail(
    State(state): State<AppState>,
    Path(record_id): Path<i32>,
) -> Result<Json<TrendDetailResponse>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let record = ai_trend_analysis::find_by_id(&mut conn, record_id)
        .map_err(|e| {
            tracing::error!("Failed to query ai analysis detail: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::NotFound)?;

    Ok(Json(to_prediction_response(&record)))
}

// ==================== 辅助类型和函数 ====================

/// 信号数据
struct SignalData {
    datetime: String,
    signal_date: NaiveDate,
    change_pct: f64,
    volume_ratio: f64,
    turnover_rate: f64,
    bid_ask_ratio: f64,
}

/// K线数据
struct KlineData {
    trade_date: NaiveDate,
    open_price: String,
    close_price: String,
    high_price: String,
    low_price: String,
    volume: i64,
}

/// 查询股票的信号数据（从 stock_snapshots）
fn query_stock_signals(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    stock_code_input: &str,
) -> Result<(Vec<SignalData>, Option<String>), AppError> {
    use bigdecimal::BigDecimal;
    use chrono::{DateTime, Utc as ChronoUtc};
    use diesel::prelude::*;
    use diesel::sql_types::{Numeric, Text, Timestamptz};

    #[derive(Debug, QueryableByName)]
    struct SnapshotSignal {
        #[diesel(sql_type = Timestamptz)]
        created_at: DateTime<ChronoUtc>,
        #[diesel(sql_type = Numeric)]
        change_pct: BigDecimal,
        #[diesel(sql_type = Numeric)]
        volume_ratio: BigDecimal,
        #[diesel(sql_type = Numeric)]
        turnover_rate: BigDecimal,
        #[diesel(sql_type = Numeric)]
        bid_ask_ratio: BigDecimal,
        #[diesel(sql_type = Text)]
        stock_name: String,
    }

    let query = r#"
        SELECT
            created_at,
            change_pct,
            volume_ratio,
            turnover_rate,
            bid_ask_ratio,
            stock_name
        FROM stock_snapshots
        WHERE stock_code = $1
        ORDER BY created_at ASC
    "#;

    let results: Vec<SnapshotSignal> = diesel::sql_query(query)
        .bind::<Text, _>(stock_code_input)
        .load(conn)
        .map_err(|e| {
            tracing::error!("Failed to query stock signals: {}", e);
            AppError::InternalServerError
        })?;

    let stock_name = results.first().map(|r| r.stock_name.clone());

    let utc_plus_8 = FixedOffset::east_opt(8 * 3600).unwrap();
    let signals: Vec<SignalData> = results
        .into_iter()
        .map(|r| {
            let local_time = r.created_at.with_timezone(&utc_plus_8);
            let signal_date = local_time.date_naive();
            let datetime = local_time.format("%Y-%m-%d %H:%M:%S%.3f %z").to_string();
            SignalData {
                datetime,
                signal_date,
                change_pct: bigdecimal_to_f64(&r.change_pct),
                volume_ratio: bigdecimal_to_f64(&r.volume_ratio),
                turnover_rate: bigdecimal_to_f64(&r.turnover_rate),
                bid_ask_ratio: bigdecimal_to_f64(&r.bid_ask_ratio),
            }
        })
        .collect();

    Ok((signals, stock_name))
}

/// 查询最早信号日之前的第N个交易日
fn find_trading_date_before(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    before_date: NaiveDate,
    count: i64,
) -> Result<NaiveDate, AppError> {
    use diesel::prelude::*;
    use diesel::sql_types::{Date, Int8};

    #[derive(Debug, QueryableByName)]
    struct DateResult {
        #[diesel(sql_type = Date)]
        trade_date: NaiveDate,
    }

    let query = r#"
        SELECT trade_date
        FROM stock_trading_calendar
        WHERE trade_date < $1 AND is_holiday = false
        ORDER BY trade_date DESC
        LIMIT $2
    "#;

    let results: Vec<DateResult> = diesel::sql_query(query)
        .bind::<Date, _>(before_date)
        .bind::<Int8, _>(count)
        .load(conn)
        .map_err(|e| {
            tracing::error!("Failed to query trading calendar: {}", e);
            AppError::InternalServerError
        })?;

    if results.is_empty() {
        // 如果交易日历完全没有数据，回退到简单日历日减30天
        tracing::warn!(
            "Trading calendar has no data before {}, falling back to 30 calendar days",
            before_date
        );
        Ok(before_date - chrono::Duration::days(30))
    } else if (results.len() as i64) < count {
        // 交易日历数据不足，以找到的最早日期为基础继续往前推
        let found_date = results.last().unwrap().trade_date;
        let days_short = count - results.len() as i64;
        // 每个交易日约 1.5 个自然日（考虑周末），额外多加几天保险
        let extra_days = (days_short as f64 * 1.5).ceil() as i64 + 2;
        tracing::info!(
            "Trading calendar only has {} of {} trading days before {}, extending {} extra calendar days from {}",
            results.len(), count, before_date, extra_days, found_date
        );
        Ok(found_date - chrono::Duration::days(extra_days))
    } else {
        Ok(results.last().unwrap().trade_date)
    }
}

/// 查询 K线数据范围
fn query_kline_range(
    conn: &mut diesel::r2d2::PooledConnection<
        diesel::r2d2::ConnectionManager<diesel::PgConnection>,
    >,
    stock_code_input: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<Vec<KlineData>, AppError> {
    use bigdecimal::BigDecimal;
    use diesel::prelude::*;
    use diesel::sql_types::{BigInt, Date, Numeric, Text};

    #[derive(Debug, QueryableByName)]
    struct KlineRow {
        #[diesel(sql_type = Date)]
        trade_date: NaiveDate,
        #[diesel(sql_type = Numeric)]
        open_price: BigDecimal,
        #[diesel(sql_type = Numeric)]
        close_price: BigDecimal,
        #[diesel(sql_type = Numeric)]
        high_price: BigDecimal,
        #[diesel(sql_type = Numeric)]
        low_price: BigDecimal,
        #[diesel(sql_type = BigInt)]
        volume: i64,
    }

    let query = r#"
        SELECT trade_date, open_price, close_price, high_price, low_price, volume
        FROM daily_klines
        WHERE stock_code = $1
          AND trade_date >= $2
          AND trade_date <= $3
        ORDER BY trade_date ASC
    "#;

    let results: Vec<KlineRow> = diesel::sql_query(query)
        .bind::<Text, _>(stock_code_input)
        .bind::<Date, _>(start)
        .bind::<Date, _>(end)
        .load(conn)
        .map_err(|e| {
            tracing::error!("Failed to query kline range: {}", e);
            AppError::InternalServerError
        })?;

    Ok(results
        .into_iter()
        .map(|r| KlineData {
            trade_date: r.trade_date,
            open_price: r.open_price.to_string(),
            close_price: r.close_price.to_string(),
            high_price: r.high_price.to_string(),
            low_price: r.low_price.to_string(),
            volume: r.volume,
        })
        .collect())
}

/// 检查K线数据是否需要补齐
/// 判断依据：数据条数少于5，或者首条K线日期晚于期望起始日期（说明前面的数据缺失）
fn needs_kline_fill(klines: &[KlineData], start: NaiveDate, _end: NaiveDate) -> bool {
    if klines.len() < 5 {
        return true;
    }
    // 如果首条K线的日期比期望起始日期晚超过3个自然日，说明前面的数据缺失
    if let Some(first) = klines.first() {
        if first.trade_date > start + chrono::Duration::days(3) {
            return true;
        }
    }
    false
}

/// 自动补齐K线数据
async fn fill_missing_klines(
    state: &AppState,
    stock_code_input: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<(), AppError> {
    let client = http_client::create_em_client().map_err(|e| {
        tracing::error!("Failed to create HTTP client: {}", e);
        AppError::InternalServerError
    })?;

    let start_str = start.format("%Y%m%d").to_string();
    let end_str = end.format("%Y%m%d").to_string();

    let kline_result =
        kline_service::fetch_and_parse_kline_data(&client, stock_code_input, &start_str, &end_str)
            .await
            .map_err(|e| {
                tracing::error!("Failed to fetch kline data for {}: {}", stock_code_input, e);
                AppError::InternalServerError
            })?;

    tracing::info!(
        "Fetched {} klines for {}, inserting into DB",
        kline_result.parsed.len(),
        stock_code_input
    );

    // 批量写入数据库
    for kline_data in &kline_result.parsed {
        let mut conn = state
            .db_pool
            .get()
            .map_err(|_| AppError::InternalServerError)?;
        match daily_kline::create(&mut conn, kline_data) {
            Ok(_) => {}
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => {
                // 已存在，跳过
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to insert kline for {} on {}: {}",
                    stock_code_input,
                    kline_data.trade_date,
                    e
                );
            }
        }
    }

    Ok(())
}

/// BigDecimal 转 f64
fn bigdecimal_to_f64(val: &bigdecimal::BigDecimal) -> f64 {
    use std::str::FromStr;
    f64::from_str(&val.to_string()).unwrap_or(0.0)
}

/// 将 DB 模型转换为 API 响应
fn to_prediction_response(record: &crate::models::AiTrendAnalysis) -> TrendPredictionResponse {
    TrendPredictionResponse {
        id: record.id,
        stock_code: record.stock_code.clone(),
        stock_name: record.stock_name.clone(),
        model_name: record.model_name.clone(),
        status: record.status.clone(),
        response_json: record.response_json.clone(),
        signal_count: record.signal_count,
        kline_start_date: record.kline_start_date,
        kline_end_date: record.kline_end_date,
        error_message: record.error_message.clone(),
        duration_ms: record.duration_ms,
        created_at: record.created_at,
    }
}
