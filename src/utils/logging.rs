use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, fmt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,axum=info"));

    // 控制台输出层（始终启用）
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true);

    // 检查是否启用文件日志（生产环境）
    let log_to_file = std::env::var("LOG_TO_FILE")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false);

    if log_to_file {
        let log_dir = std::env::var("LOG_DIR").unwrap_or_else(|_| "./logs".to_string());
        
        // 按天轮转日志文件
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            &log_dir,
            "stock-backend.log"
        );
        
        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_ansi(false)  // 文件不需要 ANSI 颜色
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .with(file_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(console_layer)
            .init();
    }
}
