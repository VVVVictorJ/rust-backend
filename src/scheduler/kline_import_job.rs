use tokio_cron_scheduler::{JobScheduler, JobBuilder};
use chrono_tz::Asia::Shanghai;
use crate::app::DbPool;
use crate::repositories::stock_snapshot;
use crate::services::kline_service;
use crate::utils::http_client;
use chrono::Local;
use tokio::time::{sleep, Duration};

/// K线导入任务执行结果
#[derive(Debug)]
pub struct KlineImportResult {
    pub total_stocks: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub stock_details: Vec<StockImportDetail>,
}

#[derive(Debug, serde::Serialize)]
pub struct StockImportDetail {
    pub stock_code: String,
    pub imported_count: usize,
    pub success: bool,
    pub error: Option<String>,
}

pub async fn create_kline_import_job(
    scheduler: &JobScheduler,
    db_pool: DbPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建每天15:01执行的任务（北京时间 UTC+8）
    // 使用 JobBuilder 设置上海时区（UTC+8）
    let job = JobBuilder::new()
        .with_timezone(Shanghai)
        .with_cron_job_type()
        .with_schedule("0 1 15 * * *")?
        .with_run_async(Box::new(move |_uuid, _l| {
            let pool = db_pool.clone();
            Box::pin(async move {
                if let Err(e) = run_kline_import_task(pool).await {
                    tracing::error!("K线导入任务失败: {}", e);
                }
            })
        }))
        .build()?;
    
    scheduler.add(job).await?;
    tracing::info!("K线导入定时任务已注册（每天北京时间 15:01 执行，使用 Asia/Shanghai 时区）");
    Ok(())
}

/// 执行K线导入任务（可以被定时任务或手动触发调用）
pub async fn run_kline_import_task(db_pool: DbPool) -> anyhow::Result<KlineImportResult> {
    tracing::info!("开始执行K线导入定时任务");
    
    let start_time = chrono::Local::now().naive_local();
    let mut history_id: Option<i32> = None;
    
    // 记录任务开始
    {
        let mut conn = db_pool.get()?;
        let new_history = crate::models::NewJobExecutionHistory {
            job_name: "kline_import".to_string(),
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
        
        match crate::repositories::job_execution_history::create(&mut conn, &new_history) {
            Ok(history) => {
                history_id = Some(history.id);
                tracing::info!("创建任务执行记录，ID: {}", history.id);
            }
            Err(e) => {
                tracing::warn!("创建任务执行记录失败: {}", e);
            }
        }
    }
    
    // 1. 获取数据库连接
    let mut conn = db_pool.get()?;
    
    // 2. 获取当天入库的股票代码
    let stock_codes = stock_snapshot::get_distinct_codes_today(&mut conn)?;
    tracing::info!("获取到 {} 个股票代码", stock_codes.len());
    
    if stock_codes.is_empty() {
        tracing::info!("今日没有股票代码入库，跳过K线导入");
        
        // 更新任务完成状态
        if let Some(id) = history_id {
            let end_time = chrono::Local::now().naive_local();
            let duration = (end_time - start_time).num_milliseconds();
            let mut conn = db_pool.get().ok();
            if let Some(ref mut c) = conn {
                let update = crate::models::UpdateJobExecutionHistory {
                    status: Some("success".to_string()),
                    completed_at: Some(end_time),
                    total_count: Some(0),
                    success_count: Some(0),
                    failed_count: Some(0),
                    skipped_count: Some(0),
                    details: None,
                    error_message: Some("今日没有股票代码入库".to_string()),
                    duration_ms: Some(duration),
                };
                let _ = crate::repositories::job_execution_history::update(c, id, &update);
            }
        }
        
        return Ok(KlineImportResult {
            total_stocks: 0,
            success_count: 0,
            failed_count: 0,
            stock_details: Vec::new(),
        });
    }
    
    // 3. 创建HTTP客户端
    let client = http_client::create_em_client()?;
    
    // 4. 获取当天日期（格式：YYYYMMDD）
    // 如果是周末，回溯到上一个交易日（周五）
    let today = get_trading_date();
    
    // 5. 遍历股票代码，批量导入K线数据
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut skipped_count = 0;
    let mut stock_details = Vec::new();
    
    // 解析交易日期用于检查
    let trade_date = chrono::NaiveDate::parse_from_str(&today, "%Y%m%d")
        .map_err(|e| anyhow::anyhow!("日期解析失败: {e}"))?;
    
    for (index, stock_code) in stock_codes.iter().enumerate() {
        // 移除前缀（SH/SZ）获取纯数字代码
        let pure_code = stock_code.trim_start_matches("SH")
            .trim_start_matches("SZ");
        
        // 先检查是否已有当天数据
        let mut conn = db_pool.get()?;
        match crate::repositories::daily_kline::exists(&mut conn, pure_code, trade_date) {
            Ok(true) => {
                // 已存在，跳过
                skipped_count += 1;
                tracing::info!("股票 {} 的 {} 数据已存在，跳过", stock_code, today);
                stock_details.push(StockImportDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: true,
                    error: Some("数据已存在，跳过导入".to_string()),
                });
                continue;
            }
            Ok(false) => {
                // 不存在，继续导入
            }
            Err(e) => {
                tracing::warn!("检查股票 {} 数据是否存在时出错: {}", stock_code, e);
                // 出错时继续尝试导入
            }
        }
        
        match import_single_stock_kline(
            &client,
            pure_code,
            &today,
            &db_pool
        ).await {
            Ok(imported) => {
                success_count += 1;
                tracing::info!("股票 {} 导入成功，导入 {} 条记录", stock_code, imported);
                stock_details.push(StockImportDetail {
                    stock_code: stock_code.clone(),
                    imported_count: imported,
                    success: true,
                    error: None,
                });
            }
            Err(e) => {
                failed_count += 1;
                let error_msg = e.to_string();
                tracing::error!("股票 {} 导入失败: {}", stock_code, error_msg);
                stock_details.push(StockImportDetail {
                    stock_code: stock_code.clone(),
                    imported_count: 0,
                    success: false,
                    error: Some(error_msg),
                });
            }
        }
        
        // 添加随机延迟，避免频繁请求被封IP（最后一个股票不需要延迟）
        if index < stock_codes.len() - 1 {
            // 随机延迟 2-5 秒
            let delay_ms = rand::random::<u64>() % 3001 + 2000; // 2000-5000ms
            tracing::debug!("等待 {} 毫秒后处理下一个股票", delay_ms);
            sleep(Duration::from_millis(delay_ms)).await;
        }
    }
    
    tracing::info!(
        "K线导入任务完成，总计: {}, 成功: {}, 失败: {}, 跳过: {}",
        stock_codes.len(),
        success_count,
        failed_count,
        skipped_count
    );
    
    // 更新任务完成状态
    if let Some(id) = history_id {
        let end_time = chrono::Local::now().naive_local();
        let duration = (end_time - start_time).num_milliseconds();
        let mut conn = db_pool.get().ok();
        if let Some(ref mut c) = conn {
            let status = if failed_count == 0 {
                "success"
            } else if success_count > 0 {
                "partial"
            } else {
                "failed"
            };
            
            let details_json = serde_json::to_value(&stock_details).ok();
            
            let update = crate::models::UpdateJobExecutionHistory {
                status: Some(status.to_string()),
                completed_at: Some(end_time),
                total_count: Some(stock_codes.len() as i32),
                success_count: Some(success_count as i32),
                failed_count: Some(failed_count as i32),
                skipped_count: Some(skipped_count as i32),
                details: details_json,
                error_message: None,
                duration_ms: Some(duration),
            };
            
            match crate::repositories::job_execution_history::update(c, id, &update) {
                Ok(_) => tracing::info!("任务执行记录已更新"),
                Err(e) => tracing::warn!("更新任务执行记录失败: {}", e),
            }
        }
    }
    
    Ok(KlineImportResult {
        total_stocks: stock_codes.len(),
        success_count,
        failed_count,
        stock_details,
    })
}

async fn import_single_stock_kline(
    client: &reqwest::Client,
    stock_code: &str,
    date: &str,
    db_pool: &DbPool,
) -> anyhow::Result<usize> {
    // 1. 从东方财富获取并解析K线数据
    let kline_result = kline_service::fetch_and_parse_kline_data(
        client,
        stock_code,
        date,
        date,
    ).await?;
    
    // 2. 获取数据库连接
    let mut conn = db_pool.get()?;
    
    // 3. 批量插入数据库
    let mut imported_count = 0;
    use crate::repositories::daily_kline;
    
    for kline_data in kline_result.parsed {
        match daily_kline::create(&mut conn, &kline_data) {
            Ok(_) => imported_count += 1,
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _
            )) => {
                // 重复数据，忽略
            }
            Err(e) => {
                tracing::warn!("插入K线数据失败: {}", e);
            }
        }
    }
    
    Ok(imported_count)
}

/// 获取交易日期：如果是周末则返回上周五，否则返回当天
fn get_trading_date() -> String {
    use chrono::{Datelike, Duration, Weekday};
    
    let now = Local::now();
    let weekday = now.weekday();
    
    let trading_date = match weekday {
        Weekday::Sat => now - Duration::days(1), // 周六 -> 上周五
        Weekday::Sun => now - Duration::days(2), // 周日 -> 上周五
        _ => now, // 工作日使用当天
    };
    
    trading_date.format("%Y%m%d").to_string()
}

