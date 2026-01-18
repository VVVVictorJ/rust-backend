use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::stock_table::{
    create_stock_table, delete_stock_table, get_stock_table, list_stock_tables, update_stock_table,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_stock_table).get(list_stock_tables))
        .route(
            "/:id",
            get(get_stock_table).put(update_stock_table).delete(delete_stock_table),
        )
}
