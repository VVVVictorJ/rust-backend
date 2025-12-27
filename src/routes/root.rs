use axum::{routing::get, Router};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
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
