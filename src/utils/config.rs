use std::net::SocketAddr;

pub struct ServerConfig {
    pub addr: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Self {
        let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = std::env::var("PORT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(8001);
        let addr: SocketAddr = format!("{}:{}", host, port)
            .parse()
            .expect("Invalid HOST/PORT");
        Self { addr }
    }
}
