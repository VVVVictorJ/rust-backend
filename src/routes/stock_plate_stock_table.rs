use axum::{
    routing::{delete, get, post},
    Router,
};

use crate::app::AppState;
use crate::handler::stock_plate_stock_table::{
    create_stock_plate_stock_table, delete_stock_plate_stock_table, query_stock_plate_stocks,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(query_stock_plate_stocks))
        .route("/", post(create_stock_plate_stock_table))
        .route("/:plate_id/:stock_table_id", delete(delete_stock_plate_stock_table))
}
