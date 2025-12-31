use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_price_compare::query_price_compare;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(query_price_compare))
}

