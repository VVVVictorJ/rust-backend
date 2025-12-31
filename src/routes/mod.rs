use axum::Router;

use crate::app::AppState;

mod root;
pub mod stock;
mod stock_request;
mod stock_snapshot;
mod profit_analysis;
mod stock_request_stock;
mod daily_kline;
mod scheduler;
mod stock_trade_date_query;

pub fn build_routes() -> Router<AppState> {
    let api_router = Router::new()
        .merge(stock::router())
        .nest("/stock-request-stocks", stock_request_stock::router())
        .nest("/stock-snapshots", stock_snapshot::router())
        .nest("/profit-analyses", profit_analysis::router())
        .nest("/daily-klines", daily_kline::router())
        .nest("/scheduler", scheduler::router())
        .nest("/stock-trade-date-query", stock_trade_date_query::router());

    Router::new()
        // 根路径与健康检查
        .merge(root::router())
        // API 路由
        .nest("/api", api_router)
        .nest("/stock-requests", stock_request::router())
}
