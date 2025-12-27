use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::profit_analysis::{create_profit_analysis, delete_profit_analysis, get_profit_analysis};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_profit_analysis))
        .route("/:id", get(get_profit_analysis).delete(delete_profit_analysis))
}

