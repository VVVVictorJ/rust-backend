use axum::{routing::get, Router};

pub fn router() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
}

async fn root() -> &'static str {
    "Axum minimal backend"
}

async fn healthz() -> &'static str {
    "ok"
}