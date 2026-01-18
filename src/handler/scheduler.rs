use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Serialize;

use crate::api_models::scheduler::{
    HistoryQueryParams, JobExecutionHistoryItem, JobExecutionHistoryResponse, JobInfo,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::repositories::job_execution_history;
use crate::scheduler::{
    kline_import_job, profit_analysis_job, stock_filter_job, stock_plate_sync_job, stock_table_sync_job,
};

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
    
    // 广播任务开始
    crate::utils::ws_broadcast::broadcast_task_status(
        &state.ws_sender,
        "kline_import".to_string(),
        "running".to_string(),
    );
    
    // 调用定时任务的核心逻辑
    match kline_import_job::run_kline_import_task(state.db_pool.clone()).await {
        Ok(result) => {
            // 广播任务完成
            let status = if result.failed_count == 0 {
                "success"
            } else if result.success_count > 0 {
                "partial"
            } else {
                "failed"
            };
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "kline_import".to_string(),
                status.to_string(),
            );
            
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
            // 广播任务失败
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "kline_import".to_string(),
                "failed".to_string(),
            );
            Err(AppError::InternalServerError)
        }
    }
}

/// 手动触发盈利分析任务
pub async fn trigger_profit_analysis(
    State(state): State<AppState>,
) -> Result<Json<TriggerProfitAnalysisResponse>, AppError> {
    tracing::info!("收到手动触发盈利分析任务的请求");
    
    // 广播任务开始
    crate::utils::ws_broadcast::broadcast_task_status(
        &state.ws_sender,
        "profit_analysis".to_string(),
        "running".to_string(),
    );
    
    // 调用定时任务的核心逻辑
    match profit_analysis_job::run_profit_analysis_task(state.db_pool.clone()).await {
        Ok(result) => {
            // 广播任务完成
            let status = if result.analyzed_count > 0 || result.skipped_count > 0 || result.total_snapshots == 0 {
                "success"
            } else {
                "failed"
            };
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "profit_analysis".to_string(),
                status.to_string(),
            );
            
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
            // 广播任务失败
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "profit_analysis".to_string(),
                "failed".to_string(),
            );
            Err(AppError::InternalServerError)
        }
    }
}

/// 股票筛选任务响应
#[derive(Serialize)]
pub struct TriggerStockFilterResponse {
    pub success: bool,
    pub message: String,
    pub items_count: usize,
}

#[derive(Serialize)]
pub struct TriggerStockTableSyncResponse {
    pub success: bool,
    pub message: String,
    pub total_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub details: Vec<StockTableSyncDetail>,
}

#[derive(Serialize)]
pub struct StockTableSyncDetail {
    pub stock_code: String,
    pub stock_name: String,
    pub action: String,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct TriggerStockPlateSyncResponse {
    pub success: bool,
    pub message: String,
    pub total_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub details: Vec<StockPlateSyncDetail>,
}

#[derive(Serialize)]
pub struct StockPlateSyncDetail {
    pub stock_code: String,
    pub plate_total: usize,
    pub plate_inserted: usize,
    pub relation_inserted: usize,
    pub action: String,
    pub error: Option<String>,
}

/// 手动触发股票筛选任务
pub async fn trigger_stock_filter(
    State(state): State<AppState>,
) -> Result<Json<TriggerStockFilterResponse>, AppError> {
    tracing::info!("收到手动触发股票筛选任务的请求");
    
    // 广播任务开始
    crate::utils::ws_broadcast::broadcast_task_status(
        &state.ws_sender,
        "stock_filter".to_string(),
        "running".to_string(),
    );
    
    // 调用定时任务的核心逻辑
    match stock_filter_job::run_stock_filter_task(state.db_pool.clone(), "manual").await {
        Ok(result) => {
            // 广播任务完成
            let status = if result.success { "success" } else { "failed" };
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_filter".to_string(),
                status.to_string(),
            );
            
            Ok(Json(TriggerStockFilterResponse {
                success: result.success,
                message: format!(
                    "股票筛选任务执行完成，筛选到 {} 只符合条件的股票",
                    result.items_count
                ),
                items_count: result.items_count,
            }))
        }
        Err(e) => {
            tracing::error!("手动触发股票筛选任务失败: {}", e);
            // 广播任务失败
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_filter".to_string(),
                "failed".to_string(),
            );
            Err(AppError::InternalServerError)
        }
    }
}

/// 手动触发 stock_table 同步任务
pub async fn trigger_stock_table_sync(
    State(state): State<AppState>,
) -> Result<Json<TriggerStockTableSyncResponse>, AppError> {
    tracing::info!("收到手动触发 stock_table 同步任务的请求");

    crate::utils::ws_broadcast::broadcast_task_status(
        &state.ws_sender,
        "stock_table_sync".to_string(),
        "running".to_string(),
    );

    match stock_table_sync_job::run_stock_table_sync_task(state.db_pool.clone()).await {
        Ok(result) => {
            let status = if result.failed_count == 0 {
                "success"
            } else if result.success_count > 0 {
                "partial"
            } else {
                "failed"
            };
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_table_sync".to_string(),
                status.to_string(),
            );

            let details = result
                .details
                .into_iter()
                .map(|d| StockTableSyncDetail {
                    stock_code: d.stock_code,
                    stock_name: d.stock_name,
                    action: d.action,
                    error: d.error,
                })
                .collect();

            Ok(Json(TriggerStockTableSyncResponse {
                success: result.failed_count == 0,
                message: format!(
                    "stock_table 同步任务执行完成，总计 {} 条，成功 {} 条，失败 {} 条，跳过 {} 条",
                    result.total_count, result.success_count, result.failed_count, result.skipped_count
                ),
                total_count: result.total_count,
                success_count: result.success_count,
                failed_count: result.failed_count,
                skipped_count: result.skipped_count,
                details,
            }))
        }
        Err(e) => {
            tracing::error!("手动触发 stock_table 同步任务失败: {}", e);
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_table_sync".to_string(),
                "failed".to_string(),
            );
            Err(AppError::InternalServerError)
        }
    }
}

/// 手动触发 stock_plate 同步任务
pub async fn trigger_stock_plate_sync(
    State(state): State<AppState>,
) -> Result<Json<TriggerStockPlateSyncResponse>, AppError> {
    tracing::info!("收到手动触发 stock_plate 同步任务的请求");

    crate::utils::ws_broadcast::broadcast_task_status(
        &state.ws_sender,
        "stock_plate_sync".to_string(),
        "running".to_string(),
    );

    match stock_plate_sync_job::run_stock_plate_sync_task(state.db_pool.clone()).await {
        Ok(result) => {
            let status = if result.failed_count == 0 {
                "success"
            } else if result.success_count > 0 {
                "partial"
            } else {
                "failed"
            };
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_plate_sync".to_string(),
                status.to_string(),
            );

            let details = result
                .details
                .into_iter()
                .map(|d| StockPlateSyncDetail {
                    stock_code: d.stock_code,
                    plate_total: d.plate_total,
                    plate_inserted: d.plate_inserted,
                    relation_inserted: d.relation_inserted,
                    action: d.action,
                    error: d.error,
                })
                .collect();

            Ok(Json(TriggerStockPlateSyncResponse {
                success: result.failed_count == 0,
                message: format!(
                    "stock_plate 同步任务执行完成，总计 {} 条，成功 {} 条，失败 {} 条，跳过 {} 条",
                    result.total_count, result.success_count, result.failed_count, result.skipped_count
                ),
                total_count: result.total_count,
                success_count: result.success_count,
                failed_count: result.failed_count,
                skipped_count: result.skipped_count,
                details,
            }))
        }
        Err(e) => {
            tracing::error!("手动触发 stock_plate 同步任务失败: {}", e);
            crate::utils::ws_broadcast::broadcast_task_status(
                &state.ws_sender,
                "stock_plate_sync".to_string(),
                "failed".to_string(),
            );
            Err(AppError::InternalServerError)
        }
    }
}

/// 获取任务列表
pub async fn get_job_list() -> Result<Json<Vec<JobInfo>>, AppError> {
    let jobs = vec![
        JobInfo {
            name: "kline_import".to_string(),
            display_name: "K线数据导入".to_string(),
            description: "自动导入当天的K线数据到数据库".to_string(),
            schedule: "每天 15:01".to_string(),
            enabled: true,
        },
        JobInfo {
            name: "stock_table_sync".to_string(),
            display_name: "stock_table 同步".to_string(),
            description: "从快照去重写入 stock_table".to_string(),
            schedule: "每天 04:00".to_string(),
            enabled: true,
        },
        JobInfo {
            name: "stock_plate_sync".to_string(),
            display_name: "stock_plate 同步".to_string(),
            description: "根据 stock_table 同步板块及关系".to_string(),
            schedule: "每天 04:30".to_string(),
            enabled: true,
        },
        JobInfo {
            name: "profit_analysis".to_string(),
            display_name: "盈利分析".to_string(),
            description: "分析股票快照的盈利情况".to_string(),
            schedule: "每天 15:40".to_string(),
            enabled: true,
        },
        JobInfo {
            name: "stock_filter_morning".to_string(),
            display_name: "股票筛选(上午)".to_string(),
            description: "交易时段自动筛选符合条件的股票并入库".to_string(),
            schedule: "工作日 9:30-12:00 每分钟".to_string(),
            enabled: true,
        },
        JobInfo {
            name: "stock_filter_afternoon".to_string(),
            display_name: "股票筛选(下午)".to_string(),
            description: "交易时段自动筛选符合条件的股票并入库".to_string(),
            schedule: "工作日 13:00-15:00 每分钟".to_string(),
            enabled: true,
        },
    ];
    
    Ok(Json(jobs))
}

/// 获取执行历史
pub async fn get_execution_history(
    Query(params): Query<HistoryQueryParams>,
    State(state): State<AppState>,
) -> Result<Json<JobExecutionHistoryResponse>, AppError> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(20);
    
    // 将空字符串转换为 None
    let job_name_filter = params.job_name.filter(|s| !s.is_empty());
    let status_filter = params.status.filter(|s| !s.is_empty());
    
    let mut conn = state.db_pool.get()
        .map_err(|_| AppError::InternalServerError)?;
    
    let (items, total) = job_execution_history::paginate(
        &mut conn,
        job_name_filter,
        status_filter,
        page,
        page_size,
    )
    .map_err(|_| AppError::InternalServerError)?;
    
    let items: Vec<JobExecutionHistoryItem> = items
        .into_iter()
        .map(|h| h.into())
        .collect();
    
    Ok(Json(JobExecutionHistoryResponse {
        total,
        page,
        page_size,
        items,
    }))
}

/// 获取历史详情
pub async fn get_execution_detail(
    Path(id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<JobExecutionHistoryItem>, AppError> {
    let mut conn = state.db_pool.get()
        .map_err(|_| AppError::InternalServerError)?;
    
    let history = job_execution_history::find_by_id(&mut conn, id)
        .map_err(|_| AppError::InternalServerError)?;
    
    Ok(Json(history.into()))
}

/// 获取最新执行记录
pub async fn get_latest_execution(
    Path(job_name): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Option<JobExecutionHistoryItem>>, AppError> {
    let mut conn = state.db_pool.get()
        .map_err(|_| AppError::InternalServerError)?;
    
    let history = job_execution_history::find_latest_by_job_name(&mut conn, &job_name)
        .map_err(|_| AppError::InternalServerError)?;
    
    Ok(Json(history.map(|h| h.into())))
}
