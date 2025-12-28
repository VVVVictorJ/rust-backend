use tokio_cron_scheduler::{Job, JobScheduler};
use crate::app::DbPool;
use crate::repositories::stock_snapshot;
use crate::services::kline_service;
use crate::utils::http_client;
use chrono::Local;

/// K线导入任务执行结果
#[derive(Debug)]
pub struct KlineImportResult {
    pub total_stocks: usize,
    pub success_count: usize,
    pub failed_count: usize,
    pub stock_details: Vec<StockImportDetail>,
}

#[derive(Debug)]
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
    // 创建每天15:01执行的任务
    let job = Job::new_async("0 1 15 * * *", move |_uuid, _l| {
        let pool = db_pool.clone();
        Box::pin(async move {
            if let Err(e) = run_kline_import_task(pool).await {
                tracing::error!("K线导入任务失败: {}", e);
            }
        })
    })?;
    
    scheduler.add(job).await?;
    Ok(())
}

/// 执行K线导入任务（可以被定时任务或手动触发调用）
pub async fn run_kline_import_task(db_pool: DbPool) -> anyhow::Result<KlineImportResult> {
    tracing::info!("开始执行K线导入定时任务");
    
    // 1. 获取数据库连接
    let mut conn = db_pool.get()?;
    
    // 2. 获取当天入库的股票代码
    let stock_codes = stock_snapshot::get_distinct_codes_today(&mut conn)?;
    tracing::info!("获取到 {} 个股票代码", stock_codes.len());
    
    if stock_codes.is_empty() {
        tracing::info!("今日没有股票代码入库，跳过K线导入");
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
    let mut stock_details = Vec::new();
    
    for stock_code in stock_codes.iter() {
        // 移除前缀（SH/SZ）获取纯数字代码
        let pure_code = stock_code.trim_start_matches("SH")
            .trim_start_matches("SZ");
        
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
    }
    
    tracing::info!(
        "K线导入任务完成，成功: {}, 失败: {}",
        success_count,
        failed_count
    );
    
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

