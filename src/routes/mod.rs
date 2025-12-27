use axum::Router;

use crate::app::AppState;

mod root;
pub mod stock;
mod stock_request;

pub fn build_routes() -> Router<AppState> {
    Router::new()
        // 根路径与健康检查
        .merge(root::router())
        // 本需求按 /stock 直挂根路径（如需 /api 可改为 .nest(\"/api\", ...)）
        .nest("/api", stock::router())
        .nest("/stock-requests", stock_request::router())
}
