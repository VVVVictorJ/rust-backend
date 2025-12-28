use tokio_cron_scheduler::{Job, JobScheduler};
use bigdecimal::BigDecimal;
use chrono::{Local, NaiveDate, Datelike, Weekday};
use std::str::FromStr;

use crate::app::DbPool;
use crate::models::{NewProfitAnalysis, StockSnapshot};
use crate::repositories::{daily_kline, profit_analysis, stock_request, stock_snapshot};

/// 盈利分析任务执行结果
#[derive(Debug)]
pub struct ProfitAnalysisResult {
    pub total_snapshots: usize,
    pub analyzed_count: usize,
    pub skipped_count: usize,
    pub no_kline_count: usize,
    pub snapshot_details: Vec<SnapshotAnalysisDetail>,
}

#[derive(Debug)]
pub struct SnapshotAnalysisDetail {
    pub stock_code: String,
    pub stock_name: String,
    pub profit_rate: i32,
    pub success: bool,
    pub error: Option<String>,
}

/// 创建盈利分析定时任务（每天15:30执行）
pub async fn create_profit_analysis_job(
    scheduler: &JobScheduler,
    db_pool: DbPool,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建每天15:30执行的任务
    let job = Job::new_async("0 30 15 * * *", move |_uuid, _l| {
        let pool = db_pool.clone();
        Box::pin(async move {
            if let Err(e) = run_profit_analysis_task(pool).await {
                tracing::error!("盈利分析任务失败: {}", e);
            }
        })
    })?;
    
    scheduler.add(job).await?;
    tracing::info!("盈利分析定时任务已注册（每天15:30执行）");
    Ok(())
}

/// 执行盈利分析任务（可以被定时任务或手动触发调用）
pub async fn run_profit_analysis_task(db_pool: DbPool) -> anyhow::Result<ProfitAnalysisResult> {
    tracing::info!("开始执行盈利分析任务");
    
    // 1. 获取数据库连接
    let mut conn = db_pool.get()?;
    
    // 2. 查找所有 time_range_end 为空的请求（待处理）
    let pending_requests = stock_request::find_pending_requests(&mut conn)?;
    tracing::info!("找到 {} 个待处理的请求", pending_requests.len());
    
    if pending_requests.is_empty() {
        tracing::info!("没有待处理的请求，跳过盈利分析");
        return Ok(ProfitAnalysisResult {
            total_snapshots: 0,
            analyzed_count: 0,
            skipped_count: 0,
            no_kline_count: 0,
            snapshot_details: Vec::new(),
        });
    }
    
    // 3. 遍历每个请求，处理其下的快照
    let mut total_snapshots = 0;
    let mut analyzed_count = 0;
    let mut skipped_count = 0;
    let mut no_kline_count = 0;
    let mut snapshot_details = Vec::new();
    
    for request in pending_requests.iter() {
        // 3.1 检查 time_range_start 是否存在
        let time_range_start = match request.time_range_start {
            Some(start) => start,
            None => {
                tracing::warn!("请求 {} 没有设置 time_range_start，跳过", request.id);
                continue;
            }
        };
        
        // 3.2 计算 K线日期 = time_range_start + 1 天（智能处理周末）
        let kline_date = get_next_trading_date(time_range_start);
        tracing::info!(
            "请求 {}: time_range_start={}, K线日期={}",
            request.id, time_range_start, kline_date
        );
        
        // 3.3 获取该请求下的所有快照
        let mut conn = db_pool.get()?;
        let snapshots = stock_snapshot::find_by_request_id(&mut conn, request.id)?;
        tracing::info!("请求 {} 下有 {} 个快照", request.id, snapshots.len());
        
        if snapshots.is_empty() {
            tracing::info!("请求 {} 没有快照，跳过", request.id);
            continue;
        }
        
        total_snapshots += snapshots.len();
        
        // 3.4 遍历快照，计算盈利指标
        for snapshot in snapshots.iter() {
            let result = analyze_single_snapshot(&db_pool, snapshot, kline_date).await;
            
            match result {
                Ok(detail) => {
                    if detail.success {
                        if detail.error.is_some() && detail.error.as_ref().unwrap().contains("已存在") {
                            skipped_count += 1;
                        } else if detail.error.is_some() && detail.error.as_ref().unwrap().contains("K线") {
                            no_kline_count += 1;
                        } else {
                            analyzed_count += 1;
                        }
                    }
                    snapshot_details.push(detail);
                }
                Err(e) => {
                    tracing::error!("分析快照 {} 失败: {}", snapshot.stock_code, e);
                    snapshot_details.push(SnapshotAnalysisDetail {
                        stock_code: snapshot.stock_code.clone(),
                        stock_name: snapshot.stock_name.clone(),
                        profit_rate: -1,
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        // 3.5 更新该请求的 time_range_end
        let today = Local::now().date_naive();
        let mut conn = db_pool.get()?;
        if let Err(e) = stock_request::update_time_range_end(&mut conn, request.id, today) {
            tracing::warn!("更新请求 {} 的 time_range_end 失败: {}", request.id, e);
        } else {
            tracing::info!("已更新请求 {} 的 time_range_end 为 {}", request.id, today);
        }
    }
    
    tracing::info!(
        "盈利分析任务完成，总计: {}, 分析: {}, 跳过: {}, 无K线: {}",
        total_snapshots,
        analyzed_count,
        skipped_count,
        no_kline_count
    );
    
    Ok(ProfitAnalysisResult {
        total_snapshots,
        analyzed_count,
        skipped_count,
        no_kline_count,
        snapshot_details,
    })
}

/// 分析单个快照的盈利情况
async fn analyze_single_snapshot(
    db_pool: &DbPool,
    snapshot: &StockSnapshot,
    trade_date: NaiveDate,
) -> anyhow::Result<SnapshotAnalysisDetail> {
    let mut conn = db_pool.get()?;
    
    // 1. 检查是否已存在分析记录
    if profit_analysis::exists_for_snapshot(&mut conn, snapshot.id, "OHLC")? {
        tracing::info!("快照 {} ({}) 已存在OHLC分析记录，跳过", snapshot.id, snapshot.stock_code);
        return Ok(SnapshotAnalysisDetail {
            stock_code: snapshot.stock_code.clone(),
            stock_name: snapshot.stock_name.clone(),
            profit_rate: -1,
            success: true,
            error: Some("分析记录已存在，跳过".to_string()),
        });
    }
    
    // 2. 获取纯股票代码（移除前缀）
    let pure_code = snapshot.stock_code
        .trim_start_matches("SH")
        .trim_start_matches("SZ");
    
    // 3. 获取今日K线数据
    let kline = match daily_kline::find_by_pk(&mut conn, pure_code, trade_date) {
        Ok(k) => k,
        Err(diesel::result::Error::NotFound) => {
            tracing::warn!("股票 {} 在 {} 没有K线数据", snapshot.stock_code, trade_date);
            return Ok(SnapshotAnalysisDetail {
                stock_code: snapshot.stock_code.clone(),
                stock_name: snapshot.stock_name.clone(),
                profit_rate: -1,
                success: true,
                error: Some(format!("{}的K线数据不存在", trade_date)),
            });
        }
        Err(e) => return Err(e.into()),
    };
    
    // 4. 计算盈利指标
    let entry_price = &snapshot.latest_price;
    let profit_high = entry_price * BigDecimal::from_str("1.10").unwrap(); // +10%
    let profit_low = entry_price * BigDecimal::from_str("1.05").unwrap();  // +5%
    
    let high_price = &kline.high_price;
    let close_price = &kline.close_price;
    
    // 判断规则
    let profit_rate = if high_price >= &profit_high && close_price >= &profit_low {
        2 // high >= profit_high 且 close >= profit_low
    } else if high_price >= &profit_low && close_price < &profit_low {
        1 // high >= profit_low 但 close < profit_low
    } else {
        0 // high < profit_low
    };
    
    tracing::info!(
        "股票 {} ({}): 入场价={}, profit_high={}, profit_low={}, K线high={}, K线close={}, profit_rate={}",
        snapshot.stock_code,
        snapshot.stock_name,
        entry_price,
        profit_high,
        profit_low,
        high_price,
        close_price,
        profit_rate
    );
    
    // 5. 写入分析结果
    let new_analysis = NewProfitAnalysis {
        snapshot_id: snapshot.id,
        strategy_name: "OHLC".to_string(),
        profit_rate: BigDecimal::from(profit_rate),
    };
    
    profit_analysis::create(&mut conn, &new_analysis)?;
    
    Ok(SnapshotAnalysisDetail {
        stock_code: snapshot.stock_code.clone(),
        stock_name: snapshot.stock_name.clone(),
        profit_rate,
        success: true,
        error: None,
    })
}

/// 获取下一个交易日期：基于给定日期 + 1 天，如果是周末则顺延到周一
fn get_next_trading_date(base_date: NaiveDate) -> NaiveDate {
    let next_day = base_date + chrono::Days::new(1);
    let weekday = next_day.weekday();
    
    match weekday {
        Weekday::Sat => next_day + chrono::Days::new(2), // 周六 -> 周一
        Weekday::Sun => next_day + chrono::Days::new(1), // 周日 -> 周一
        _ => next_day, // 工作日直接使用
    }
}

