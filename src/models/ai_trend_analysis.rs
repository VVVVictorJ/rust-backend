use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use serde_json::Value as JsonValue;

use crate::schema::ai_trend_analysis;

#[derive(Queryable, Debug, Clone)]
#[allow(dead_code)]
pub struct AiTrendAnalysis {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub model_name: String,
    pub status: String,
    pub request_payload: JsonValue,
    pub response_json: Option<JsonValue>,
    pub raw_response: Option<String>,
    pub signal_count: Option<i32>,
    pub kline_start_date: Option<NaiveDate>,
    pub kline_end_date: Option<NaiveDate>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = ai_trend_analysis)]
pub struct NewAiTrendAnalysis {
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub model_name: String,
    pub status: String,
    pub request_payload: JsonValue,
    pub response_json: Option<JsonValue>,
    pub raw_response: Option<String>,
    pub signal_count: Option<i32>,
    pub kline_start_date: Option<NaiveDate>,
    pub kline_end_date: Option<NaiveDate>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

#[derive(AsChangeset, Debug, Clone)]
#[diesel(table_name = ai_trend_analysis)]
pub struct UpdateAiTrendAnalysis {
    pub status: Option<String>,
    pub response_json: Option<JsonValue>,
    pub raw_response: Option<String>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub updated_at: Option<DateTime<Utc>>,
}
