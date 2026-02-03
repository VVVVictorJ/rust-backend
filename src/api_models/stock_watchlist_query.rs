use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use bigdecimal::BigDecimal;

use super::stock_trade_date_query::PlateInfo;

/// 观察表查询请求
#[derive(Debug, Deserialize)]
pub struct WatchlistQueryRequest {
    /// 板块代码列表（多选，可选）
    #[serde(default)]
    pub plate_codes: Vec<String>,
    /// 涨跌幅最小值（可选）
    pub change_pct_min: Option<BigDecimal>,
    /// 涨跌幅最大值（可选）
    pub change_pct_max: Option<BigDecimal>,
    /// 量比最小值（可选）
    pub volume_ratio_min: Option<BigDecimal>,
    /// 量比最大值（可选）
    pub volume_ratio_max: Option<BigDecimal>,
    /// 换手率最小值（可选）
    pub turnover_rate_min: Option<BigDecimal>,
    /// 换手率最大值（可选）
    pub turnover_rate_max: Option<BigDecimal>,
    /// 委比最小值（可选）
    pub bid_ask_ratio_min: Option<BigDecimal>,
    /// 委比最大值（可选）
    pub bid_ask_ratio_max: Option<BigDecimal>,
    /// 主力资金流入最小值（可选）
    pub main_force_inflow_min: Option<BigDecimal>,
    /// 主力资金流入最大值（可选）
    pub main_force_inflow_max: Option<BigDecimal>,
    /// 股票代码模糊匹配（可选）
    pub stock_code_filter: Option<String>,
}

/// 观察表查询结果项
#[derive(Debug, Serialize)]
pub struct WatchlistQueryItem {
    pub stock_code: String,
    pub stock_name: Option<String>,
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

/// 观察表查询响应
#[derive(Debug, Serialize)]
pub struct WatchlistQueryResponse {
    /// 数据列表
    pub data: Vec<WatchlistQueryItem>,
    /// 总记录数
    pub total: i64,
}

/// 观察表明细查询请求
#[derive(Debug, Deserialize)]
pub struct WatchlistDetailRequest {
    /// 股票代码
    pub stock_code: String,
}

/// 观察表明细查询结果项
#[derive(Debug, Serialize)]
pub struct WatchlistDetailItem {
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

/// 观察表明细查询响应
#[derive(Debug, Serialize)]
pub struct WatchlistDetailResponse {
    /// 数据列表
    pub data: Vec<WatchlistDetailItem>,
    /// 总记录数
    pub total: i64,
}

/// 观察表K线查询请求
#[derive(Debug, Deserialize)]
pub struct WatchlistKlineRequest {
    /// 股票代码
    pub stock_code: String,
}

/// 观察表K线查询结果项
#[derive(Debug, Serialize)]
pub struct WatchlistKlineItem {
    pub stock_code: String,
    pub trade_date: NaiveDate,
    pub open_price: BigDecimal,
    pub high_price: BigDecimal,
    pub low_price: BigDecimal,
    pub close_price: BigDecimal,
    pub volume: i64,
    pub amount: BigDecimal,
}

/// 观察表K线查询响应
#[derive(Debug, Serialize)]
pub struct WatchlistKlineResponse {
    /// 数据列表
    pub data: Vec<WatchlistKlineItem>,
    /// 总记录数
    pub total: i64,
    /// 起始日期（该股票在 stock_snapshots 中首次出现的日期）
    pub start_date: Option<NaiveDate>,
    /// 结束日期（当前日期）
    pub end_date: NaiveDate,
}

/// 补齐观察表K线数据请求
#[derive(Debug, Deserialize)]
pub struct WatchlistFillKlineRequest {
    // 可以为空，表示补齐所有观察表中的股票
}

/// 补齐观察表K线数据响应
#[derive(Debug, Serialize)]
pub struct WatchlistFillKlineResponse {
    /// 总股票数
    pub total_stocks: usize,
    /// 成功数
    pub success_count: usize,
    /// 失败数
    pub failed_count: usize,
    /// 跳过数（无快照数据或无需补齐）
    pub skipped_count: usize,
    /// 股票详情
    pub stock_details: Vec<StockFillKlineDetail>,
}

/// 股票补齐K线数据详情
#[derive(Debug, Serialize)]
pub struct StockFillKlineDetail {
    /// 股票代码
    pub stock_code: String,
    /// 导入的K线数据条数
    pub imported_count: usize,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}
