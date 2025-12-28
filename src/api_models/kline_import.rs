use serde::{Deserialize, Serialize};

/// K线导入请求参数
#[derive(Debug, Deserialize)]
pub struct ImportKlineRequest {
    pub stock_code: String,
    /// 开始日期，格式: YYYYMMDD (如 "20251226")
    pub start_date: String,
    /// 结束日期，格式: YYYYMMDD (如 "20251227")
    pub end_date: String,
}

/// K线导入响应
#[derive(Debug, Serialize)]
pub struct ImportKlineResponse {
    pub success: bool,
    pub stock_code: String,
    pub stock_name: String,
    pub total_count: usize,
    pub imported_count: usize,
    pub failed_count: usize,
    pub errors: Vec<String>,
}

