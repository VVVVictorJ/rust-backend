use axum::{routing::{get, post}, Router};

use crate::app::AppState;
use crate::handler::stock_plate::{
    create_stock_plate, delete_stock_plate, get_stock_plate, list_stock_plates, update_stock_plate,
};
use crate::handler::stock_plate_em::fetch_em_stock_plates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/em", get(fetch_em_stock_plates))
        .route("/", post(create_stock_plate).get(list_stock_plates))
        .route(
            "/:id",
            get(get_stock_plate).put(update_stock_plate).delete(delete_stock_plate),
        )
}
