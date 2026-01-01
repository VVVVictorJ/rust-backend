use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::scheduler::{
    get_execution_detail, get_execution_history, get_job_list, get_latest_execution,
    trigger_kline_import, trigger_profit_analysis,
};
use crate::handler::ws_handler;

pub fn router() -> Router<AppState> {
    Router::new()
        // 手动触发接口
        .route("/trigger-kline-import", post(trigger_kline_import))
        .route("/trigger-profit-analysis", post(trigger_profit_analysis))
        // 查询接口
        .route("/jobs", get(get_job_list))
        .route("/history", get(get_execution_history))
        .route("/history/:id", get(get_execution_detail))
        .route("/latest/:job_name", get(get_latest_execution))
        // WebSocket 路由
        .route("/ws", get(ws_handler::ws_handler))
}

