use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

/// 价格对比查询请求
#[derive(Debug, Deserialize)]
pub struct PriceCompareRequest {
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

/// 价格对比查询结果项
#[derive(Debug, Serialize)]
pub struct PriceCompareItem {
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub high_price: BigDecimal,
    pub close_price: BigDecimal,
    pub open_price: BigDecimal,
    pub low_price: BigDecimal,
    pub grade: String,
    pub created_at: DateTime<Utc>,
}

/// 价格对比查询响应（分页）
#[derive(Debug, Serialize)]
pub struct PriceCompareResponse {
    /// 数据列表
    pub data: Vec<PriceCompareItem>,
    /// 总记录数
    pub total: i64,
    /// 当前页码
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
    /// 快照日期（前一个交易日）
    pub snapshot_date: Option<NaiveDate>,
    /// 查询的交易日期
    pub trade_date: Option<NaiveDate>,
}

