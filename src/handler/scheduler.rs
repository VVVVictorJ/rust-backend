use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::app::AppState;
use crate::handler::error::AppError;
use crate::scheduler::{kline_import_job, profit_analysis_job};

#[derive(Serialize)]
pub struct TriggerTaskResponse {
    pub success: bool,
    pub message: String,
    pub total_stocks: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub details: Vec<StockDetail>,
}

#[derive(Serialize)]
pub struct StockDetail {
    pub stock_code: String,
    pub imported_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// 盈利分析任务响应
#[derive(Serialize)]
pub struct TriggerProfitAnalysisResponse {
    pub success: bool,
    pub message: String,
    pub total_snapshots: usize,
    pub analyzed_count: usize,
    pub skipped_count: usize,
    pub no_kline_count: usize,
    pub details: Vec<SnapshotDetail>,
}

#[derive(Serialize)]
pub struct SnapshotDetail {
    pub stock_code: String,
    pub stock_name: String,
    pub profit_rate: i32,
    pub success: bool,
    pub error: Option<String>,
}

/// 手动触发K线导入任务
pub async fn trigger_kline_import(
    State(state): State<AppState>,
) -> Result<Json<TriggerTaskResponse>, AppError> {
    tracing::info!("收到手动触发K线导入任务的请求");
    
    // 调用定时任务的核心逻辑
    match kline_import_job::run_kline_import_task(state.db_pool.clone()).await {
        Ok(result) => {
            let details = result.stock_details.into_iter()
                .map(|d| StockDetail {
                    stock_code: d.stock_code,
                    imported_count: d.imported_count,
                    success: d.success,
                    error: d.error,
                })
                .collect();
            
            Ok(Json(TriggerTaskResponse {
                success: result.failed_count == 0,
                message: format!(
                    "K线导入任务执行完成，总计 {} 只股票，成功 {} 只，失败 {} 只",
                    result.total_stocks, result.success_count, result.failed_count
                ),
                total_stocks: result.total_stocks,
                success_count: result.success_count,
                failed_count: result.failed_count,
                details,
            }))
        }
        Err(e) => {
            tracing::error!("手动触发K线导入任务失败: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}

/// 手动触发盈利分析任务
pub async fn trigger_profit_analysis(
    State(state): State<AppState>,
) -> Result<Json<TriggerProfitAnalysisResponse>, AppError> {
    tracing::info!("收到手动触发盈利分析任务的请求");
    
    // 调用定时任务的核心逻辑
    match profit_analysis_job::run_profit_analysis_task(state.db_pool.clone()).await {
        Ok(result) => {
            let details = result.snapshot_details.into_iter()
                .map(|d| SnapshotDetail {
                    stock_code: d.stock_code,
                    stock_name: d.stock_name,
                    profit_rate: d.profit_rate,
                    success: d.success,
                    error: d.error,
                })
                .collect();
            
            Ok(Json(TriggerProfitAnalysisResponse {
                success: result.analyzed_count > 0 || result.skipped_count > 0 || result.total_snapshots == 0,
                message: format!(
                    "盈利分析任务执行完成，总计 {} 个快照，分析 {} 个，跳过 {} 个，无K线 {} 个",
                    result.total_snapshots, result.analyzed_count, result.skipped_count, result.no_kline_count
                ),
                total_snapshots: result.total_snapshots,
                analyzed_count: result.analyzed_count,
                skipped_count: result.skipped_count,
                no_kline_count: result.no_kline_count,
                details,
            }))
        }
        Err(e) => {
            tracing::error!("手动触发盈利分析任务失败: {}", e);
            Err(AppError::InternalServerError)
        }
    }
}

