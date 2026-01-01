use axum::Router;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use tower_http::trace::{
    DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
};
use tracing::Level;

use crate::routes;
use crate::utils::middleware;
use crate::utils::ws_broadcast::TaskStatusSender;

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub ws_sender: TaskStatusSender,
}

#[allow(dead_code)]
pub fn build_app() -> Router {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let db_pool = Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool");
    let ws_sender = crate::utils::ws_broadcast::create_broadcast_channel();
    let state = AppState { db_pool, ws_sender };

    routes::build_routes()
        .with_state(state)
        .layer(middleware::cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
}

pub fn build_app_with_pool(db_pool: DbPool, ws_sender: TaskStatusSender) -> Router {
    let state = AppState { db_pool, ws_sender };

    routes::build_routes()
        .with_state(state)
        .layer(middleware::cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        )
}