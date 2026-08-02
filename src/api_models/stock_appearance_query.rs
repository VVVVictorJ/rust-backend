use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::stock_trade_date_query::PlateInfo;

/// 历史出现查询请求
#[derive(Debug, Deserialize)]
pub struct StockAppearanceQueryRequest {
    /// 股票代码（可选，模糊匹配）
    pub stock_code: Option<String>,
    /// 股票名称（可选，模糊匹配）
    pub stock_name: Option<String>,
    /// 板块代码（可选，精确匹配）
    pub plate_code: Option<String>,
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

/// 历史出现查询结果项
#[derive(Debug, Serialize)]
pub struct StockAppearanceQueryItem {
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

/// 历史出现查询响应（分页）
#[derive(Debug, Serialize)]
pub struct StockAppearanceQueryResponse {
    /// 数据列表
    pub data: Vec<StockAppearanceQueryItem>,
    /// 总记录数
    pub total: i64,
    /// 当前页码
    pub page: i64,
    /// 每页数量
    pub page_size: i64,
    /// 总页数
    pub total_pages: i64,
}
