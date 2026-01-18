use chrono::Local;
use chrono_tz::Asia::Shanghai;
use serde::Serialize;
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::app::DbPool;
use crate::models::{NewJobExecutionHistory, NewStockTable, UpdateJobExecutionHistory};
use crate::repositories::{job_execution_history, stock_snapshot, stock_table};
use crate::utils::ws_broadcast::TaskStatusSender;

#[derive(Debug, Serialize)]
pub struct StockTableSyncDetail {
    pub stock_code: String,
    pub stock_name: String,
    pub action: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct StockTableSyncResult {
    pub total_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub skipped_count: usize,
    pub details: Vec<StockTableSyncDetail>,
}

/// 创建 stock_table 同步任务（每天 UTC+8 04:00 执行）
pub async fn create_stock_table_sync_job(
    scheduler: &JobScheduler,
    db_pool: DbPool,
    ws_sender: TaskStatusSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let job = JobBuilder::new()
        .with_timezone(Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 4 * * *")?
        .with_run_async(Box::new(move |_uuid, _l| {
            let pool = db_pool.clone();
            let sender = ws_sender.clone();
            Box::pin(async move {
                crate::utils::ws_broadcast::broadcast_task_status(
                    &sender,
                    "stock_table_sync".to_string(),
                    "running".to_string(),
                );
                match run_stock_table_sync_task(pool).await {
                    Ok(result) => {
                        let status = if result.failed_count == 0 {
                            "success"
                        } else if result.success_count > 0 {
                            "partial"
                        } else {
                            "failed"
                        };
                        crate::utils::ws_broadcast::broadcast_task_status(
                            &sender,
                            "stock_table_sync".to_string(),
                            status.to_string(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("stock_table 同步任务失败: {}", e);
                        crate::utils::ws_broadcast::broadcast_task_status(
                            &sender,
                            "stock_table_sync".to_string(),
                            "failed".to_string(),
                        );
                    }
                }
            })
        }))
        .build()?;

    scheduler.add(job).await?;
    tracing::info!("stock_table 同步定时任务已注册（每天北京时间 04:00 执行，使用 Asia/Shanghai 时区）");
    Ok(())
}

pub async fn run_stock_table_sync_task(db_pool: DbPool) -> anyhow::Result<StockTableSyncResult> {
    tracing::info!("开始执行 stock_table 同步任务");
    let start_time = Local::now().naive_local();
    let mut history_id: Option<i32> = None;

    {
        let mut conn = db_pool.get()?;
        let new_history = NewJobExecutionHistory {
            job_name: "stock_table_sync".to_string(),
            status: "running".to_string(),
            started_at: start_time,
            completed_at: None,
            total_count: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            details: None,
            error_message: None,
            duration_ms: None,
        };
        if let Ok(history) = job_execution_history::create(&mut conn, &new_history) {
            history_id = Some(history.id);
            tracing::debug!("创建任务执行记录，ID: {}", history.id);
        }
    }

    let mut conn = db_pool.get()?;
    let distinct = stock_snapshot::list_distinct_codes_with_name(&mut conn)?;
    if distinct.is_empty() {
        tracing::info!("没有快照数据，跳过同步");
        if let Some(id) = history_id {
            let end_time = Local::now().naive_local();
            let duration = (end_time - start_time).num_milliseconds();
            let update = UpdateJobExecutionHistory {
                status: Some("success".to_string()),
                completed_at: Some(end_time),
                total_count: Some(0),
                success_count: Some(0),
                failed_count: Some(0),
                skipped_count: Some(0),
                details: None,
                error_message: Some("没有快照数据".to_string()),
                duration_ms: Some(duration),
            };
            if let Ok(mut c) = db_pool.get() {
                let _ = job_execution_history::update(&mut c, id, &update);
            }
        }
        return Ok(StockTableSyncResult {
            total_count: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            details: Vec::new(),
        });
    }

    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut details = Vec::with_capacity(distinct.len());

    for item in distinct {
        if stock_table::exists_by_code(&mut conn, &item.code)? {
            skipped_count += 1;
            details.push(StockTableSyncDetail {
                stock_code: item.code,
                stock_name: item.name,
                action: "skipped".to_string(),
                error: None,
            });
            continue;
        }

        let new_stock = NewStockTable {
            stock_code: item.code.clone(),
            stock_name: item.name.clone(),
        };
        match stock_table::create(&mut conn, &new_stock) {
            Ok(_) => {
                success_count += 1;
                details.push(StockTableSyncDetail {
                    stock_code: item.code,
                    stock_name: item.name,
                    action: "inserted".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                failed_count += 1;
                details.push(StockTableSyncDetail {
                    stock_code: item.code,
                    stock_name: item.name,
                    action: "failed".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let total_count = success_count + failed_count + skipped_count;
    tracing::info!(
        "stock_table 同步完成，总计: {}, 成功: {}, 失败: {}, 跳过: {}",
        total_count,
        success_count,
        failed_count,
        skipped_count
    );

    if let Some(id) = history_id {
        let end_time = Local::now().naive_local();
        let duration = (end_time - start_time).num_milliseconds();
        let status = if failed_count == 0 {
            "success"
        } else if success_count > 0 {
            "partial"
        } else {
            "failed"
        };
        let details_json = serde_json::to_value(&details).ok();
        let update = UpdateJobExecutionHistory {
            status: Some(status.to_string()),
            completed_at: Some(end_time),
            total_count: Some(total_count as i32),
            success_count: Some(success_count as i32),
            failed_count: Some(failed_count as i32),
            skipped_count: Some(skipped_count as i32),
            details: details_json,
            error_message: None,
            duration_ms: Some(duration),
        };
        if let Ok(mut c) = db_pool.get() {
            let _ = job_execution_history::update(&mut c, id, &update);
        }
    }

    Ok(StockTableSyncResult {
        total_count,
        success_count,
        failed_count,
        skipped_count,
        details,
    })
}
