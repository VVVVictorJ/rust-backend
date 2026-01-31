use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use super::stock_trade_date_query::PlateInfo;

/// 动态回溯查询请求
#[derive(Debug, Deserialize)]
pub struct DynamicBacktrackRequest {
    /// 基准交易日期，格式：YYYY-MM-DD
    pub trade_date: String,
    /// 往回多少个交易日（包括当天）
    pub trade_days: i32,
    /// 最少出现次数
    pub min_occurrences: i32,
}

/// 动态回溯查询结果项
#[derive(Debug, Serialize)]
pub struct DynamicBacktrackItem {
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
    /// 出现次数
    pub occurrence_count: i32,
    /// 板块信息
    pub plates: Vec<PlateInfo>,
}

/// 动态回溯查询响应
#[derive(Debug, Serialize)]
pub struct DynamicBacktrackResponse {
    /// 数据列表
    pub data: Vec<DynamicBacktrackItem>,
    /// 总记录数
    pub total: i64,
}

/// 动态回溯明细查询请求
#[derive(Debug, Deserialize)]
pub struct DynamicBacktrackDetailRequest {
    /// 股票代码
    pub stock_code: String,
    /// 基准交易日期，格式：YYYY-MM-DD
    pub trade_date: String,
    /// 往回多少个交易日（包括当天）
    pub trade_days: i32,
}

/// 动态回溯明细查询响应（复用 TrackDetailResponse）
pub use super::stock_track_query::{TrackDetailItem, TrackDetailResponse};

