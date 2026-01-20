use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::stock_trade_date_query::{query_by_trade_date, refresh_missing_plates};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(query_by_trade_date))
        .route("/refresh-plates", post(refresh_missing_plates))
}

