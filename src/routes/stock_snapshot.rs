use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::stock_snapshot::{create_stock_snapshot, delete_stock_snapshot, get_stock_snapshot};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_stock_snapshot))
        .route("/:id", get(get_stock_snapshot).delete(delete_stock_snapshot))
}

