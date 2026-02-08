use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// ==================== 趋势预测请求 ====================

/// 趋势预测分析请求
#[derive(Debug, Deserialize)]
pub struct TrendPredictionRequest {
    /// 股票代码
    pub stock_code: String,
}

// ==================== 趋势预测响应 ====================

/// 趋势预测分析响应
#[derive(Debug, Serialize)]
pub struct TrendPredictionResponse {
    /// 分析记录ID
    pub id: i32,
    /// 股票代码
    pub stock_code: String,
    /// 股票名称
    pub stock_name: Option<String>,
    /// AI模型名称
    pub model_name: String,
    /// 分析状态
    pub status: String,
    /// AI返回的结构化JSON结果
    pub response_json: Option<JsonValue>,
    /// 信号数量
    pub signal_count: Option<i32>,
    /// K线起始日期
    pub kline_start_date: Option<NaiveDate>,
    /// K线结束日期
    pub kline_end_date: Option<NaiveDate>,
    /// 错误信息
    pub error_message: Option<String>,
    /// 分析耗时(毫秒)
    pub duration_ms: Option<i64>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
}

// ==================== 历史记录查询 ====================

/// 历史分析记录查询请求
#[derive(Debug, Deserialize)]
pub struct TrendHistoryRequest {
    /// 股票代码（可选，不传则查所有）
    pub stock_code: Option<String>,
    /// 分页: 每页条数
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    /// 分页: 页码（从1开始）
    #[serde(default = "default_page")]
    pub page: i64,
}

fn default_page_size() -> i64 {
    20
}

fn default_page() -> i64 {
    1
}

/// 历史分析记录列表项
#[derive(Debug, Serialize)]
pub struct TrendHistoryItem {
    pub id: i32,
    pub stock_code: String,
    pub stock_name: Option<String>,
    pub model_name: String,
    pub status: String,
    pub signal_count: Option<i32>,
    pub kline_start_date: Option<NaiveDate>,
    pub kline_end_date: Option<NaiveDate>,
    pub duration_ms: Option<i64>,
    pub created_at: DateTime<Utc>,
}

/// 历史分析记录响应
#[derive(Debug, Serialize)]
pub struct TrendHistoryResponse {
    pub data: Vec<TrendHistoryItem>,
    pub total: i64,
}

// ==================== 详情查询 ====================

/// 分析详情响应（与 TrendPredictionResponse 相同）
pub type TrendDetailResponse = TrendPredictionResponse;
