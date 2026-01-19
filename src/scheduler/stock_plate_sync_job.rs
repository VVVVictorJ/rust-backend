use chrono::Local;
use chrono_tz::Asia::Shanghai;
use rand::Rng;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tokio::time::{sleep, Duration};
use tokio_cron_scheduler::{JobBuilder, JobScheduler};

use crate::app::DbPool;
use crate::models::{
    NewJobExecutionHistory, NewStockPlate, NewStockPlateStockTable, UpdateJobExecutionHistory,
    UpdateStockPlate,
};
use crate::repositories::{job_execution_history, stock_plate, stock_plate_stock_table, stock_table};
use crate::services::stock_plate_em::fetch_em_plate_list;
use crate::utils::http_client::create_em_client;
use crate::utils::ws_broadcast::TaskStatusSender;

#[derive(Debug, Serialize)]
pub struct StockPlateSyncDetail {
    pub stock_code: String,
    pub plate_total: usize,
    pub plate_inserted: usize,
    pub plate_updated: usize,
    pub relation_inserted: usize,
    pub relation_deleted: usize,
    pub action: String,
    pub error: Option<String>,
}

#[derive(Debug)]
pub struct StockPlateSyncResult {
    #[allow(dead_code)]
    pub total_count: usize,
    pub success_count: usize,
    pub failed_count: usize,
    #[allow(dead_code)]
    pub skipped_count: usize,
    #[allow(dead_code)]
    pub details: Vec<StockPlateSyncDetail>,
}

/// 创建 stock_plate 同步任务（每天 UTC+8 18:00 执行）
pub async fn create_stock_plate_sync_job(
    scheduler: &JobScheduler,
    db_pool: DbPool,
    ws_sender: TaskStatusSender,
) -> Result<(), Box<dyn std::error::Error>> {
    let job = JobBuilder::new()
        .with_timezone(Shanghai)
        .with_cron_job_type()
        .with_schedule("0 0 18 * * *")?
        .with_run_async(Box::new(move |_uuid, _l| {
            let pool = db_pool.clone();
            let sender = ws_sender.clone();
            Box::pin(async move {
                crate::utils::ws_broadcast::broadcast_task_status(
                    &sender,
                    "stock_plate_sync".to_string(),
                    "running".to_string(),
                );
                match run_stock_plate_sync_task(pool).await {
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
                            "stock_plate_sync".to_string(),
                            status.to_string(),
                        );
                    }
                    Err(e) => {
                        tracing::error!("stock_plate 同步任务失败: {}", e);
                        crate::utils::ws_broadcast::broadcast_task_status(
                            &sender,
                            "stock_plate_sync".to_string(),
                            "failed".to_string(),
                        );
                    }
                }
            })
        }))
        .build()?;

    scheduler.add(job).await?;
    tracing::info!("stock_plate 同步定时任务已注册（每天北京时间 18:00 执行，使用 Asia/Shanghai 时区）");
    Ok(())
}

pub async fn run_stock_plate_sync_task(db_pool: DbPool) -> anyhow::Result<StockPlateSyncResult> {
    tracing::info!("开始执行 stock_plate 同步任务");
    let start_time = Local::now().naive_local();
    let mut history_id: Option<i32> = None;

    {
        let mut conn = db_pool.get()?;
        let new_history = NewJobExecutionHistory {
            job_name: "stock_plate_sync".to_string(),
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

    let stocks = {
        let mut conn = db_pool.get()?;
        stock_table::list_all(&mut conn)?
    };

    if stocks.is_empty() {
        tracing::info!("stock_table 为空，跳过同步");
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
                error_message: Some("stock_table 为空".to_string()),
                duration_ms: Some(duration),
            };
            if let Ok(mut c) = db_pool.get() {
                let _ = job_execution_history::update(&mut c, id, &update);
            }
        }
        return Ok(StockPlateSyncResult {
            total_count: 0,
            success_count: 0,
            failed_count: 0,
            skipped_count: 0,
            details: Vec::new(),
        });
    }

    let client = create_em_client()?;
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut details = Vec::with_capacity(stocks.len());
    let mut consecutive_errors: u32 = 0;

    for stock in stocks {
        let base_delay_ms = rand::thread_rng().gen_range(450..=900);
        let extra_delay_ms = if consecutive_errors > 0 {
            let capped = consecutive_errors.min(5) as u64;
            (capped * 400) + rand::thread_rng().gen_range(0..=200) as u64
        } else {
            0
        };
        let delay_ms = base_delay_ms as u64 + extra_delay_ms;
        sleep(Duration::from_millis(delay_ms)).await;

        let request_start = Instant::now();
        tracing::info!("请求板块信息: stock_code={}", stock.stock_code);
        let response = fetch_em_plate_list(&client, &stock.stock_code).await;
        match response {
            Ok(res) => {
                consecutive_errors = 0;
                tracing::info!(
                    "板块请求完成: stock_code={}, plates={}, elapsed_ms={}",
                    stock.stock_code,
                    res.total,
                    request_start.elapsed().as_millis()
                );
                if res.items.is_empty() {
                    skipped_count += 1;
                    details.push(StockPlateSyncDetail {
                        stock_code: stock.stock_code,
                        plate_total: 0,
                        plate_inserted: 0,
                        plate_updated: 0,
                        relation_inserted: 0,
                        relation_deleted: 0,
                        action: "skipped".to_string(),
                        error: None,
                    });
                    continue;
                }

                let mut conn = db_pool.get()?;
                let mut plate_inserted = 0;
                let mut plate_updated = 0;
                let mut relation_inserted = 0;
                let mut relation_deleted = 0;
                let mut has_changes = false;

                let existing_relations =
                    stock_plate_stock_table::list_by_stock_table_id(&mut conn, stock.id)?;
                let mut existing_map: HashMap<String, stock_plate_stock_table::StockPlateRelationInfo> =
                    existing_relations
                        .into_iter()
                        .map(|rel| (rel.plate_code.clone(), rel))
                        .collect();
                let mut latest_codes: HashSet<String> = HashSet::new();

                for item in res.items {
                    latest_codes.insert(item.plate_code.clone());
                    let existing_plate = stock_plate::find_by_plate_code(&mut conn, &item.plate_code)?;
                    let plate = if let Some(mut plate) = existing_plate {
                        if plate.name != item.name {
                            let update = UpdateStockPlate {
                                plate_code: None,
                                name: Some(item.name.clone()),
                                updated_at: Some(Local::now().naive_local()),
                            };
                            if stock_plate::update_by_id(&mut conn, plate.id, &update).is_ok() {
                                plate.name = item.name.clone();
                                plate_updated += 1;
                                has_changes = true;
                            }
                        }
                        if let Some(existing_rel) = existing_map.get_mut(&item.plate_code) {
                            existing_rel.plate_name = plate.name.clone();
                        }
                        plate
                    } else if let Some(mut plate) = stock_plate::find_by_name(&mut conn, &item.name)? {
                        if plate.plate_code != item.plate_code || plate.name != item.name {
                            let update = UpdateStockPlate {
                                plate_code: Some(item.plate_code.clone()),
                                name: Some(item.name.clone()),
                                updated_at: Some(Local::now().naive_local()),
                            };
                            if stock_plate::update_by_id(&mut conn, plate.id, &update).is_ok() {
                                let old_code = plate.plate_code.clone();
                                plate.plate_code = item.plate_code.clone();
                                plate.name = item.name.clone();
                                plate_updated += 1;
                                has_changes = true;
                                if let Some(mut rel) = existing_map.remove(&old_code) {
                                    rel.plate_code = plate.plate_code.clone();
                                    rel.plate_name = plate.name.clone();
                                    existing_map.insert(plate.plate_code.clone(), rel);
                                }
                            }
                        }
                        plate
                    } else {
                        let new_plate = NewStockPlate {
                            plate_code: item.plate_code.clone(),
                            name: item.name.clone(),
                        };
                        match stock_plate::create(&mut conn, &new_plate) {
                            Ok(inserted) => {
                                plate_inserted += 1;
                                has_changes = true;
                                inserted
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "插入 stock_plate 失败: code={}, name={}, err={}",
                                    item.plate_code,
                                    item.name,
                                    e
                                );
                                if let Some(plate) = stock_plate::find_by_name(&mut conn, &item.name)? {
                                    plate
                                } else {
                                    continue;
                                }
                            }
                        }
                    };

                    let exists = stock_plate_stock_table::exists_by_ids(
                        &mut conn,
                        plate.id,
                        stock.id,
                    )?;
                    if !exists {
                        let new_rel = NewStockPlateStockTable {
                            plate_id: plate.id,
                            stock_table_id: stock.id,
                        };
                        if stock_plate_stock_table::create(&mut conn, &new_rel).is_ok() {
                            relation_inserted += 1;
                            has_changes = true;
                        }
                    }
                }

                for (plate_code, rel) in existing_map.iter() {
                    if !latest_codes.contains(plate_code)
                        && stock_plate_stock_table::delete_by_pk(&mut conn, rel.plate_id, stock.id)
                            .is_ok()
                    {
                        relation_deleted += 1;
                        has_changes = true;
                    }
                }

                let action = if has_changes {
                    success_count += 1;
                    "success"
                } else {
                    skipped_count += 1;
                    "no_change"
                };
                details.push(StockPlateSyncDetail {
                    stock_code: stock.stock_code,
                    plate_total: res.total as usize,
                    plate_inserted,
                    plate_updated,
                    relation_inserted,
                    relation_deleted,
                    action: action.to_string(),
                    error: None,
                });
            }
            Err(e) => {
                consecutive_errors = consecutive_errors.saturating_add(1);
                tracing::warn!(
                    "板块请求失败: stock_code={}, elapsed_ms={}, error={}",
                    stock.stock_code,
                    request_start.elapsed().as_millis(),
                    e
                );
                let cooldown_ms = rand::thread_rng().gen_range(800..=1500);
                sleep(Duration::from_millis(cooldown_ms)).await;
                failed_count += 1;
                details.push(StockPlateSyncDetail {
                    stock_code: stock.stock_code,
                    plate_total: 0,
                    plate_inserted: 0,
                    plate_updated: 0,
                    relation_inserted: 0,
                    relation_deleted: 0,
                    action: "failed".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }
    }

    let total_count = success_count + failed_count + skipped_count;
    tracing::info!(
        "stock_plate 同步完成，总计: {}, 成功: {}, 失败: {}, 跳过: {}",
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

    Ok(StockPlateSyncResult {
        total_count,
        success_count,
        failed_count,
        skipped_count,
        details,
    })
}
