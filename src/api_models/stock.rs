use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct FilteredStockItem {
    pub f57: String,   // 代码
    pub f58: String,   // 名称
    pub f43: Option<f64>, // 最新价
    pub f170: Option<f64>, // 涨跌幅
    pub f50: Option<f64>, // 量比
    pub f168: Option<f64>, // 换手率
    pub f191: Option<f64>, // 委比
    pub f137: Option<f64>, // 主力净流入
}

