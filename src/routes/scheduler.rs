use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::scheduler::trigger_kline_import;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trigger-kline-import", post(trigger_kline_import))
}

