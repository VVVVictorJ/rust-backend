use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::monthly_kline::monthly_kline_query;

pub fn router() -> Router<AppState> {
    Router::new().route("/query", post(monthly_kline_query))
}
