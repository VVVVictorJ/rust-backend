use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router{
    Router::new().route("/cat", get(get_cat))
}

#[derive(Serialize)]
struct CatResponse<'a> {
    name: &'a str,
    age: u8,
    color: &'a str,
}

async fn get_cat() -> Json<CatResponse<'static>> {
    Json(CatResponse {
        name: "Whiskers",
        age: 2,
        color: "Tabby",
    })
}