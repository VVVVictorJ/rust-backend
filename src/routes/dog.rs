use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/dog", get(get_dog))
}

#[derive(Serialize)]
struct DogResponse<'a> {
    name:  &'a str,
    breed: &'a str,
    age:   u8,
}

async fn get_dog() -> Json<DogResponse<'static>> {
    // mock 数据
    Json(DogResponse {
        name: "Buddy",
        breed: "Golden Retriever",
        age: 3,
    })
}