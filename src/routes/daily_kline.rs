use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::daily_kline::{
    create_daily_kline, delete_daily_kline, get_daily_kline,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_daily_kline))
        .route("/:stock_code/:trade_date", get(get_daily_kline).delete(delete_daily_kline))
}

