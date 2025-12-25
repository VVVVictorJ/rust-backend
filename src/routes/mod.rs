use axum::Router;

mod root;
pub mod stock;

pub fn build_routes() -> Router {
    Router::new()
        // 根路径与健康检查
        .merge(root::router())
        // 本需求按 /stock 直挂根路径（如需 /api 可改为 .nest(\"/api\", ...)）
        .merge(stock::router())
}