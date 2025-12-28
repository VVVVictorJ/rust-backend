use axum::{
    extract::State,
    Json,
};
use serde::Serialize;

use crate::app::AppState;
use crate::handler::error::AppError;
use crate::scheduler::kline_import_job;

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

