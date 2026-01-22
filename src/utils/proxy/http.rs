use std::sync::Arc;

use reqwest::header::HeaderMap;
use reqwest::Url;
use serde_json::Value;
use tokio::sync::Mutex;

use super::{ProxyClient, ProxyError};

pub async fn proxy_get_json(
    proxy_client: &Arc<Mutex<ProxyClient>>,
    url: Url,
    headers: &HeaderMap,
) -> Result<Value, ProxyError> {
    let max_attempts = 2;
    let mut current_url = url;
    let mut attempted_http_fallback = false;
    for attempt in 1..=max_attempts {
        let client = {
            let mut guard = proxy_client.lock().await;
            guard.get_client().await?
        };
        let resp = match client.get(current_url.clone()).headers(headers.clone()).send().await {
            Ok(resp) => resp,
            Err(err) => {
                let error_debug = format!("{err:?}");
                let is_incomplete = error_debug.contains("IncompleteMessage");
                if is_incomplete && !attempted_http_fallback && current_url.scheme() == "https" {
                    let mut http_url = current_url.clone();
                    if http_url.set_scheme("http").is_ok() {
                        attempted_http_fallback = true;
                        current_url = http_url;
                        continue;
                    }
                }

                {
                    let mut guard = proxy_client.lock().await;
                    guard.invalidate_proxy();
                }
                if attempt < max_attempts {
                    continue;
                }
                return Err(err.into());
            }
        };
        let json = match resp.json::<Value>().await {
            Ok(json) => json,
            Err(err) => {
                if attempt < max_attempts {
                    continue;
                }
                return Err(err.into());
            }
        };
        return Ok(json);
    }
    Err(ProxyError::NoProxyData)
}
