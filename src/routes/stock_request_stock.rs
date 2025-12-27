use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_request_stock::create_stock_request_stock;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(create_stock_request_stock))
}

