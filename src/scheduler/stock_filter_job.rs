use tokio_cron_scheduler::{JobScheduler, JobBuilder};
use chrono_tz::Asia::Shanghai;
use crate::app::DbPool;
use crate::models::{NewJobExecutionHistory, NewStockRequest, NewStockSnapshot, UpdateJobExecutionHistory};
use crate::repositories::{job_execution_history, stock_request, stock_snapshot};
use crate::services::stock_filter::{get_filtered_stocks_param, FilterParams};
use crate::utils::http_client;
use crate::utils::bigdecimal_parser::parse_bigdecimal;
use serde_json::Value;

/// 股票筛选任务执行结果
#[derive(Debug)]
pub struct StockFilterResult {
    pub items_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

/// 创建股票筛选定时任务（上午和下午两个时段）
pub async fn create_stock_filter_jobs(
    scheduler: &JobScheduler,
    db_pool: DbPool,
    ws_sender: crate::utils::ws_broadcast::TaskStatusSender,
) -> Result<(), Box<dyn std::error::Error>> {
    // 上午时段 cron 表达式（工作日 1-5）
    let morning_crons = vec![
        "0 30-59 9 * * 1-5",   // 9:30-9:59
        "0 * 10-11 * * 1-5",  // 10:00-11:59
        "0 0 12 * * 1-5",     // 12:00
    ];
    
    // 下午时段 cron 表达式（工作日 1-5）
    let afternoon_crons = vec![
        "0 * 13-14 * * 1-5",  // 13:00-14:59
        "0 0 15 * * 1-5",     // 15:00
    ];
    
    // 注册上午时段任务
    for cron_expr in morning_crons {
        let pool = db_pool.clone();
        let sender = ws_sender.clone();
        
        let job = JobBuilder::new()
            .with_timezone(Shanghai)
            .with_cron_job_type()
            .with_schedule(cron_expr)?
            .with_run_async(Box::new(move |_uuid, _l| {
                let pool = pool.clone();
                let sender = sender.clone();
                Box::pin(async move {
                    execute_stock_filter_task(pool, sender, "morning").await;
                })
            }))
            .build()?;
        
        scheduler.add(job).await?;
        tracing::info!("股票筛选上午任务已注册: {} (Asia/Shanghai)", cron_expr);
    }
    
    // 注册下午时段任务
    for cron_expr in afternoon_crons {
        let pool = db_pool.clone();
        let sender = ws_sender.clone();
        
        let job = JobBuilder::new()
            .with_timezone(Shanghai)
            .with_cron_job_type()
            .with_schedule(cron_expr)?
            .with_run_async(Box::new(move |_uuid, _l| {
                let pool = pool.clone();
                let sender = sender.clone();
                Box::pin(async move {
                    execute_stock_filter_task(pool, sender, "afternoon").await;
                })
            }))
            .build()?;
        
        scheduler.add(job).await?;
        tracing::info!("股票筛选下午任务已注册: {} (Asia/Shanghai)", cron_expr);
    }
    
    tracing::info!("股票筛选定时任务已全部注册（上午 9:30-12:00，下午 13:00-15:00，每分钟执行）");
    Ok(())
}

/// 执行股票筛选任务的包装函数
async fn execute_stock_filter_task(
    db_pool: DbPool,
    ws_sender: crate::utils::ws_broadcast::TaskStatusSender,
    session: &str,
) {
    // 广播任务开始
    crate::utils::ws_broadcast::broadcast_task_status(
        &ws_sender,
        "stock_filter".to_string(),
        "running".to_string(),
    );
    
    match run_stock_filter_task(db_pool, session).await {
        Ok(result) => {
            let status = if result.success { "success" } else { "failed" };
            crate::utils::ws_broadcast::broadcast_task_status(
                &ws_sender,
                "stock_filter".to_string(),
                status.to_string(),
            );
        }
        Err(e) => {
            tracing::error!("股票筛选任务失败: {}", e);
            crate::utils::ws_broadcast::broadcast_task_status(
                &ws_sender,
                "stock_filter".to_string(),
                "failed".to_string(),
            );
        }
    }
}

/// 执行股票筛选任务（可被定时任务或手动触发调用）
pub async fn run_stock_filter_task(db_pool: DbPool, session: &str) -> anyhow::Result<StockFilterResult> {
    let now = chrono::Local::now();
    tracing::info!("开始执行股票筛选定时任务 [{}] - {}", session, now.format("%Y-%m-%d %H:%M:%S"));
    
    let start_time = now.naive_local();
    let mut history_id: Option<i32> = None;
    
    // 记录任务开始
    {
        let mut conn = db_pool.get()?;
        let new_history = NewJobExecutionHistory {
            job_name: format!("stock_filter_{}", session),
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
        
        match job_execution_history::create(&mut conn, &new_history) {
            Ok(history) => {
                history_id = Some(history.id);
                tracing::debug!("创建任务执行记录，ID: {}", history.id);
            }
            Err(e) => {
                tracing::warn!("创建任务执行记录失败: {}", e);
            }
        }
    }
    
    // 使用默认参数
    let params = FilterParams::default();
    
    // 创建 HTTP 客户端
    let client = http_client::create_em_client()?;
    
    // 调用筛选服务
    let result = get_filtered_stocks_param(&client, params).await;
    
    let (items_count, success, error_msg) = match result {
        Ok(ref json_result) => {
            let items = json_result.get("items").and_then(|v| v.as_array());
            
            if let Some(items_arr) = items {
                if !items_arr.is_empty() {
                    // 持久化到数据库
                    if let Err(e) = persist_to_db(&db_pool, items_arr).await {
                        tracing::warn!("持久化股票数据失败: {}", e);
                        (items_arr.len(), true, Some(format!("数据获取成功但持久化失败: {}", e)))
                    } else {
                        tracing::info!("成功筛选并持久化 {} 条股票数据", items_arr.len());
                        (items_arr.len(), true, None)
                    }
                } else {
                    tracing::info!("本次筛选没有符合条件的股票");
                    (0, true, None)
                }
            } else {
                (0, true, None)
            }
        }
        Err(e) => {
            let error_str = e.to_string();
            tracing::error!("股票筛选失败: {}", error_str);
            (0, false, Some(error_str))
        }
    };
    
    // 更新任务完成状态
    if let Some(id) = history_id {
        let end_time = chrono::Local::now().naive_local();
        let duration = (end_time - start_time).num_milliseconds();
        
        if let Ok(mut conn) = db_pool.get() {
            let status = if success { "success" } else { "failed" };
            
            let update = UpdateJobExecutionHistory {
                status: Some(status.to_string()),
                completed_at: Some(end_time),
                total_count: Some(items_count as i32),
                success_count: Some(if success { items_count as i32 } else { 0 }),
                failed_count: Some(if success { 0 } else { 1 }),
                skipped_count: Some(0),
                details: None,
                error_message: error_msg.clone(),
                duration_ms: Some(duration),
            };
            
            match job_execution_history::update(&mut conn, id, &update) {
                Ok(_) => tracing::debug!("任务执行记录已更新"),
                Err(e) => tracing::warn!("更新任务执行记录失败: {}", e),
            }
        }
    }
    
    Ok(StockFilterResult {
        items_count,
        success,
        error: error_msg,
    })
}

/// 将筛选结果持久化到数据库
async fn persist_to_db(db_pool: &DbPool, items: &[Value]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = db_pool.get()?;

    // 1. 插入 stock_requests 记录
    let now_date = chrono::Utc::now().date_naive();
    let new_request = NewStockRequest {
        strategy_name: Some("filtered_param".to_string()),
        time_range_start: Some(now_date),
        time_range_end: None,
    };
    let created_request = stock_request::create(&mut conn, &new_request)?;
    let request_id = created_request.id;

    // 2. 遍历 items，插入 stock_snapshots
    for item in items {
        let stock_code = item
            .get("f57")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let stock_name = item
            .get("f58")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let latest_price = parse_bigdecimal(item.get("f43"));
        let change_pct = parse_bigdecimal(item.get("f170"));
        let volume_ratio = parse_bigdecimal(item.get("f50"));
        let turnover_rate = parse_bigdecimal(item.get("f168"));
        let bid_ask_ratio = parse_bigdecimal(item.get("f191"));
        let main_force_inflow = parse_bigdecimal(item.get("f137"));

        let new_snapshot = NewStockSnapshot {
            request_id,
            stock_code,
            stock_name,
            latest_price,
            change_pct,
            volume_ratio,
            turnover_rate,
            bid_ask_ratio,
            main_force_inflow,
        };

        if let Err(e) = stock_snapshot::create(&mut conn, &new_snapshot) {
            tracing::warn!("插入快照失败: {}", e);
        }
    }

    Ok(())
}
