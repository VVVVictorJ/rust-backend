use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::basic_data_analysis::query_plate_statistics;

pub fn router() -> Router<AppState> {
    Router::new().route("/plate-statistics", post(query_plate_statistics))
}
