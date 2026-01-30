use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use super::stock_trade_date_query::PlateInfo;

/// 追踪查询请求
#[derive(Debug, Deserialize)]
pub struct TrackQueryRequest {
    /// 交易日期，格式：YYYY-MM-DD
    pub trade_date: String,
    /// 最少出现次数，默认3
    #[serde(default = "default_min_occurrences")]
    pub min_occurrences: i32,
}

fn default_min_occurrences() -> i32 {
    3
}

/// 股票出现次数统计
#[derive(Debug, Serialize)]
pub struct OccurrenceStats {
    /// 3天内出现次数
    pub days_3: i32,
    /// 7天内出现次数
    pub days_7: i32,
    /// 14天内出现次数
    pub days_14: i32,
}

/// 追踪查询结果项
#[derive(Debug, Serialize)]
pub struct TrackQueryItem {
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub close_price: Option<BigDecimal>,
    pub change_pct: BigDecimal,
    pub volume_ratio: BigDecimal,
    pub turnover_rate: BigDecimal,
    pub bid_ask_ratio: BigDecimal,
    pub main_force_inflow: BigDecimal,
    pub created_at: DateTime<Utc>,
    /// 出现次数统计
    pub occurrence_stats: OccurrenceStats,
    /// 标签，如"14天3次"
    pub tag: String,
    /// 板块信息
    pub plates: Vec<PlateInfo>,
}

/// 追踪查询响应
#[derive(Debug, Serialize)]
pub struct TrackQueryResponse {
    /// 数据列表
    pub data: Vec<TrackQueryItem>,
    /// 总记录数
    pub total: i64,
}

/// 追踪明细查询请求
#[derive(Debug, Deserialize)]
pub struct TrackDetailRequest {
    /// 股票代码
    pub stock_code: String,
    /// 交易日期，格式：YYYY-MM-DD
    pub trade_date: String,
    /// 追踪天数：3/7/14
    pub track_days: i32,
}

/// 追踪明细查询结果项
#[derive(Debug, Serialize)]
pub struct TrackDetailItem {
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub close_price: Option<BigDecimal>,
    pub change_pct: BigDecimal,
    pub volume_ratio: BigDecimal,
    pub turnover_rate: BigDecimal,
    pub bid_ask_ratio: BigDecimal,
    pub main_force_inflow: BigDecimal,
    pub created_at: DateTime<Utc>,
    pub plates: Vec<PlateInfo>,
}

/// 追踪明细查询响应
#[derive(Debug, Serialize)]
pub struct TrackDetailResponse {
    /// 数据列表
    pub data: Vec<TrackDetailItem>,
    /// 总记录数
    pub total: i64,
}
