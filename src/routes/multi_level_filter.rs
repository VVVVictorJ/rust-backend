use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::multi_level_filter::monthly_ma_cross_screen;

pub fn router() -> Router<AppState> {
    Router::new().route("/monthly-ma-cross", post(monthly_ma_cross_screen))
}
