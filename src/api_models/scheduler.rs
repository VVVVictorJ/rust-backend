use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::models::job_execution_history::JobExecutionHistory;

/// 任务信息
#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub schedule: String,
    pub enabled: bool,
}

/// 查询参数
#[derive(Deserialize, Debug)]
pub struct HistoryQueryParams {
    pub job_name: Option<String>,
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

/// 执行历史响应
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobExecutionHistoryResponse {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub items: Vec<JobExecutionHistoryItem>,
}

/// 执行历史条目
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct JobExecutionHistoryItem {
    pub id: i32,
    pub job_name: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub total_count: i32,
    pub success_count: i32,
    pub failed_count: i32,
    pub skipped_count: i32,
    pub details: Option<Value>,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
}

impl From<JobExecutionHistory> for JobExecutionHistoryItem {
    fn from(history: JobExecutionHistory) -> Self {
        Self {
            id: history.id,
            job_name: history.job_name,
            status: history.status,
            started_at: history.started_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            completed_at: history.completed_at.map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string()),
            total_count: history.total_count,
            success_count: history.success_count,
            failed_count: history.failed_count,
            skipped_count: history.skipped_count,
            details: history.details,
            error_message: history.error_message,
            duration_ms: history.duration_ms,
        }
    }
}

