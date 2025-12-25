use axum::http::HeaderValue;
use tower_http::cors::{Any, CorsLayer};

pub fn cors_layer() -> CorsLayer {
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
