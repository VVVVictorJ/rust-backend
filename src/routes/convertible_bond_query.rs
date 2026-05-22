use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::app::AppState;
use crate::handler::convertible_bond_query::query_convertible_bonds;

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/convertible-bond-query",
        get(|| async { Json(json!({ "ok": true, "route": "convertible-bond-query" })) })
            .post(query_convertible_bonds),
    )
}
