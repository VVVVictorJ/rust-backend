use reqwest::{Client, header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, REFERER, ACCEPT_ENCODING}};

/// 创建用于东方财富 API 的 HTTP 客户端
/// 包含必要的请求头模拟浏览器访问
pub fn create_em_client() -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        ),
    );
    headers.insert(ACCEPT, HeaderValue::from_static("application/json, text/plain, */*"));
    headers.insert(REFERER, HeaderValue::from_static("https://quote.eastmoney.com"));
    headers.insert(ACCEPT_ENCODING, HeaderValue::from_static("gzip"));
    
    Client::builder()
        .default_headers(headers)
        .build()
}

