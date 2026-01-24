mod app;
mod api_models;
mod handler;
mod repositories;
mod models;
mod schema;
mod routes;
mod utils;
mod services;
mod scheduler;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    utils::logging::init_logging();

    let cfg = utils::config::ServerConfig::from_env();
    let addr: SocketAddr = cfg.addr;
    
    // 构建应用并获取 db_pool
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let manager = diesel::r2d2::ConnectionManager::<diesel::pg::PgConnection>::new(database_url);
    let db_pool_max: u32 = std::env::var("DB_POOL_MAX")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(20);
    let db_pool_min: Option<u32> = std::env::var("DB_POOL_MIN").ok().and_then(|v| v.parse().ok());
    let mut pool_builder = diesel::r2d2::Pool::builder().max_size(db_pool_max);
    if let Some(min_idle) = db_pool_min {
        pool_builder = pool_builder.min_idle(Some(min_idle));
    }
    let db_pool = pool_builder.build(manager).expect("Failed to create DB pool");
    
    // 启动定时调度器
    let scheduler = tokio_cron_scheduler::JobScheduler::new().await.expect("创建调度器失败");
    
    // 创建 WebSocket 广播通道
    let ws_sender = utils::ws_broadcast::create_broadcast_channel();
    
    if let Err(e) = scheduler::kline_import_job::create_kline_import_job(&scheduler, db_pool.clone(), ws_sender.clone()).await {
        tracing::error!("创建K线导入任务失败: {}", e);
    }
    
    if let Err(e) = scheduler::profit_analysis_job::create_profit_analysis_job(&scheduler, db_pool.clone(), ws_sender.clone()).await {
        tracing::error!("创建盈利分析任务失败: {}", e);
    }
    
    if let Err(e) = scheduler::stock_filter_job::create_stock_filter_jobs(&scheduler, db_pool.clone(), ws_sender.clone()).await {
        tracing::error!("创建股票筛选定时任务失败: {}", e);
    }

    if let Err(e) = scheduler::stock_table_sync_job::create_stock_table_sync_job(&scheduler, db_pool.clone(), ws_sender.clone()).await {
        tracing::error!("创建 stock_table 同步定时任务失败: {}", e);
    }

    if let Err(e) = scheduler::stock_plate_sync_job::create_stock_plate_sync_job(&scheduler, db_pool.clone(), ws_sender.clone()).await {
        tracing::error!("创建 stock_plate 同步定时任务失败: {}", e);
    }
    
    scheduler.start().await.expect("启动调度器失败");
    tracing::info!("定时任务调度器已启动");
    
    // 构建并启动 Web 服务
    let app = app::build_app_with_pool(db_pool, ws_sender);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    tracing::info!(
        "Axum listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.expect("server failed");
}
