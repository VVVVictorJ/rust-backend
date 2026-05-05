use axum::{routing::post, Router};

use crate::app::AppState;
use crate::handler::convertible_bond_query::query_convertible_bonds;

pub fn router() -> Router<AppState> {
    Router::new().route("/", post(query_convertible_bonds))
}
