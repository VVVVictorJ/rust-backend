use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ConvertibleBondItem {
    pub bond_code: String,
    pub bond_short_name: String,
    pub stock_code: String,
    pub stock_name: String,
    pub issue_scale: f64,
    pub transfer_premium_ratio: f64,
    pub stock_price: Option<f64>,
    pub bond_price: Option<f64>,
    /// DELIST_DATE 非空且「最后交易日」距今天在 [0,3] 个日历日内时为 true（前端标红）
    #[serde(default)]
    pub near_last_trading_day: bool,
}

#[derive(Debug, Serialize)]
pub struct ConvertibleBondQueryResponse {
    pub data: Vec<ConvertibleBondItem>,
    pub total: i64,
}
