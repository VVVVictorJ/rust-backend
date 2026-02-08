use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::ai_analysis::{trend_prediction, trend_history, trend_detail};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trend-prediction", post(trend_prediction))
        .route("/trend-prediction/history", get(trend_history))
        .route("/trend-prediction/:id", get(trend_detail))
}
