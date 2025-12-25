use axum::{routing::get, Json, Router};
use serde::Serialize;
use std::{
    net::SocketAddr,
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // 允许使用 .env 覆盖本地开发配置
    let _ = dotenvy::dotenv();

    // 日志初始化（支持 RUST_LOG 环境变量）
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info,axum=info")),
        )
        .try_init();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8001);

    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Invalid HOST/PORT");

    let app = Router::new()
        .route("/", get(root))
        .route("/healthz", get(healthz))
        .route("/api/hello", get(api_hello))
        .route("/api/time", get(api_time))
        .layer(cors_layer())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO))
                .on_failure(DefaultOnFailure::new().level(Level::ERROR)),
        );

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    tracing::info!("Axum listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.expect("server failed");
}

async fn root() -> &'static str {
    "Axum minimal backend"
}

async fn healthz() -> &'static str {
    "ok"
}

#[derive(Serialize)]
struct HelloResponse<'a> {
    message: &'a str,
}

async fn api_hello() -> Json<HelloResponse<'static>> {
    Json(HelloResponse { message: "hello, axum" })
}

#[derive(Serialize)]
struct TimeResponse {
    epoch_ms: u128,
}

async fn api_time() -> Json<TimeResponse> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    Json(TimeResponse {
        epoch_ms: now.as_millis(),
    })
}

fn cors_layer() -> CorsLayer {
    use axum::http::HeaderValue;
    // 从环境变量解析允许的源，逗号分隔
    let allowed_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    let from_env: Vec<HeaderValue> = allowed_env
        .split(',')
        .filter_map(|o| {
            let trimmed = o.trim();
            if trimmed.is_empty() {
                None
            } else {
                HeaderValue::from_str(trimmed).ok()
            }
        })
        .collect();

    if !from_env.is_empty() {
        CorsLayer::new()
            .allow_origin(from_env)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin([
                HeaderValue::from_static("http://localhost:5173"),
                HeaderValue::from_static("http://127.0.0.1:5173"),
            ])
            .allow_methods(Any)
            .allow_headers(Any)
    }
}


