mod app;
mod handler;
mod models;
mod routes;
mod utils;
mod services;

use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv();
    utils::logging::init_logging();

    let cfg = utils::config::ServerConfig::from_env();
    let addr: SocketAddr = cfg.addr;
    let app = app::build_app();

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    tracing::info!(
        "Axum listening on http://{}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.expect("server failed");
}
