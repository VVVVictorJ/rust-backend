use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// 板块信息
#[derive(Debug, Deserialize, Serialize)]
pub struct PlateInfo {
    pub plate_code: String,
    pub name: String,
}

/// 交易日期查询请求
#[derive(Debug, Deserialize)]
pub struct TradeDateQueryRequest {
    /// 交易日期，格式：YYYY-MM-DD
    pub trade_date: String,
    /// 页码，从1开始
    #[serde(default = "default_page")]
    pub page: i64,
    /// 每页数量
    #[serde(default = "default_page_size")]
    pub page_size: i64,
}

fn default_page() -> i64 {
    1
}

fn default_page_size() -> i64 {
    20
}

/// 交易日期查询结果项
#[derive(Debug, Serialize)]
pub struct TradeDateQueryItem {
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

/// 交易日期查询响应（分页）
#[derive(Debug, Serialize)]
pub struct TradeDateQueryResponse {
    /// 数据列表
    pub data: Vec<TradeDateQueryItem>,
    /// 总记录数
    pub total: i64,
    /// 当前页码
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}

