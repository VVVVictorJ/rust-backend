use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_appearance_query::query_appearances;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(query_appearances))
}
