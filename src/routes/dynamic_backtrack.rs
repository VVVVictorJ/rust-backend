use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::dynamic_backtrack::{query_dynamic_backtrack, query_dynamic_backtrack_detail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(query_dynamic_backtrack))
        .route("/detail", post(query_dynamic_backtrack_detail))
}
