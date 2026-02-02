use axum::{routing::{get, post, delete}, Router};

use crate::app::AppState;
use crate::handler::stock_watchlist::{
    add_to_watchlist, batch_check_watchlist, check_watchlist, list_watchlist,
    remove_from_watchlist,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(add_to_watchlist).get(list_watchlist))
        .route("/check/:stock_code", get(check_watchlist))
        .route("/batch-check", post(batch_check_watchlist))
        .route("/:stock_code", delete(remove_from_watchlist))
}
