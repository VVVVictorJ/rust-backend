use std::sync::Arc;

use reqwest::header::HeaderMap;
use reqwest::Url;
use serde_json::Value;
use tokio::sync::Mutex;

use super::{ProxyClient, ProxyError};

/// 截取 UTF-8 预览（有损），便于日志排障且不刷爆控制台。
fn body_preview_utf8(bytes: &[u8], max_chars: usize) -> String {
    let s = String::from_utf8_lossy(bytes);
    s.chars().take(max_chars).collect()
}

async fn invalidate_shared_proxy(proxy_client: &Arc<Mutex<ProxyClient>>) {
    let mut guard = proxy_client.lock().await;
    guard.invalidate_proxy();
}

/// GET 并经代理出站，期望 JSON。**先读完整字节再解码**，区别于 `resp.json()`，以便：
/// - HTTP 502/网关页等非 JSON：记录状态码与正文前缀并重试/换代理；
/// - 偶发截断、压缩流异常：换代理后重试。
pub async fn proxy_get_json(
    proxy_client: &Arc<Mutex<ProxyClient>>,
    url: Url,
    headers: &HeaderMap,
) -> Result<Value, ProxyError> {
    let max_attempts = 6;
    let mut current_url = url;
    let mut attempted_http_fallback = false;
    for attempt in 1..=max_attempts {
        let client = {
            let mut guard = proxy_client.lock().await;
            guard.get_client().await?
        };
        let resp = match client
            .get(current_url.clone())
            .headers(headers.clone())
            .send()
            .await
        {
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

                invalidate_shared_proxy(proxy_client).await;
                if attempt < max_attempts {
                    continue;
                }
                return Err(err.into());
            }
        };

        let status = resp.status();
        let bytes = match resp.bytes().await {
            Ok(b) => b,
            Err(err) => {
                invalidate_shared_proxy(proxy_client).await;
                if attempt < max_attempts {
                    continue;
                }
                return Err(err.into());
            }
        };

        if !status.is_success() {
            let preview = body_preview_utf8(&bytes, 400);
            tracing::warn!(
                target: "proxy",
                %status,
                len = bytes.len(),
                preview,
                url = %current_url,
                attempt,
                "proxy_get_json upstream HTTP non-success"
            );
            invalidate_shared_proxy(proxy_client).await;
            if attempt < max_attempts {
                continue;
            }
            return Err(ProxyError::Status {
                status,
                body: preview,
            });
        }

        match serde_json::from_slice::<Value>(&bytes) {
            Ok(json) => return Ok(json),
            Err(err) => {
                let preview = body_preview_utf8(&bytes, 800);
                tracing::warn!(
                    target: "proxy",
                    attempt,
                    len = bytes.len(),
                    preview,
                    url = %current_url,
                    parse_err = %err,
                    "proxy_get_json JSON decode failed, will retry with fresh proxy if possible"
                );
                invalidate_shared_proxy(proxy_client).await;
                if attempt < max_attempts {
                    continue;
                }
                return Err(ProxyError::Parse(format!(
                    "JSON 解码失败: {err}；响应长度 {} 字节，前缀: {}",
                    bytes.len(),
                    preview
                )));
            }
        }
    }
    Err(ProxyError::NoProxyData)
}
