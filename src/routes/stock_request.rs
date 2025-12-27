use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::stock_request::{
    create_stock_request, delete_stock_request, get_stock_request,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_stock_request))
        .route("/:id", get(get_stock_request).delete(delete_stock_request))
}

