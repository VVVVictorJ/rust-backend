use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::stock_request_stock::{
    create_stock_request_stock, delete_stock_request_stock, get_stock_request_stock,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_stock_request_stock))
        .route("/:request_id/:stock_code", get(get_stock_request_stock).delete(delete_stock_request_stock))
}

