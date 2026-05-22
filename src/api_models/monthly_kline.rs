use serde::{Deserialize, Serialize};

use super::daily_kline::DailyKlineResponse;

#[derive(Debug, Deserialize)]
pub struct MonthlyKlineQueryRequest {
    pub stock_code: String,
}

#[derive(Debug, Serialize)]
pub struct MonthlyKlineQueryResponse {
    pub stock_code: String,
    pub stock_name: String,
    pub total_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_errors: Option<Vec<String>>,
    pub klines: Vec<DailyKlineResponse>,
}
