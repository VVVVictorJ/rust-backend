use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_track_query::{query_stock_track_detail, query_tracked_stocks};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(query_tracked_stocks))
        .route("/detail", post(query_stock_track_detail))
}
