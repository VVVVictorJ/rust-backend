use axum::Router;

mod root;
mod stock;
mod dog;
mod cat;

pub fn build_routes() -> Router {
    Router::new()
        // 根路径与健康检查
        .merge(root::router())
        // 业务 API 统一挂在 /api 前缀下
        .nest(
          "/api", 
        stock::router()
                      .merge(dog::router())
                      .merge(cat::router())
        )
}