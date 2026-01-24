use reqwest::header::HeaderMap;
use reqwest::{Client, Url};
use serde_json::Value;
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use rand::Rng;
use std::sync::Arc;

use crate::api_models::stock_plate_em::{EmPlateItem, EmPlateResponse};
use crate::utils::proxy::{proxy_get_json, shared_proxy_client, ProxyClient, ProxyError};
use crate::utils::secid::code_to_secid;

const EM_PLATE_URL: &str = "https://push2.eastmoney.com/api/qt/slist/get";

#[derive(Debug, Error)]
pub enum EmPlateError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("proxy error: {0}")]
    Proxy(#[from] ProxyError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing data field")]
    MissingData,
    #[error("url parse error: {0}")]
    Url(String),
}

pub async fn fetch_em_plate_list(
    _client: &Client,
    stock_code: &str,
) -> Result<EmPlateResponse, EmPlateError> {
    let proxy_client = shared_proxy_client()?;

    fetch_em_plate_list_with_proxy_client(proxy_client, _client, stock_code).await
}

pub async fn fetch_em_plate_list_with_proxy_client(
    proxy_client: Arc<Mutex<ProxyClient>>,
    _client: &Client,
    stock_code: &str,
) -> Result<EmPlateResponse, EmPlateError> {
    let ut = "fa5fd1943c7b386f172d6893dbfba10b";
    let fields = "f14,f12";
    let secid = code_to_secid(stock_code);
    let fltt = "1";
    let invt = "2";
    let pi = "0";
    let po = "1";
    let np = "1";
    let pz = "500";
    let spt = "3";
    let wbp2u = "|0|0|0|web";
    let timestamp = chrono::Utc::now().timestamp_millis().to_string();
    let headers = HeaderMap::new();

    let mut attempt = 0;
    let max_attempts = 3;
    let json: Value = loop {
        attempt += 1;
        let url = Url::parse_with_params(
            EM_PLATE_URL,
            [
                ("fltt", fltt),
                ("invt", invt),
                ("fields", fields),
                ("secid", secid.as_str()),
                ("ut", ut),
                ("pi", pi),
                ("po", po),
                ("np", np),
                ("pz", pz),
                ("spt", spt),
                ("wbp2u", wbp2u),
                ("_", timestamp.as_str()),
            ],
        )
        .map_err(|err| EmPlateError::Url(err.to_string()))?;

        match proxy_get_json(&proxy_client, url, &headers).await {
            Ok(json) => break json,
            Err(e) => {
                if attempt < max_attempts {
                    let backoff = 200_u64.saturating_mul(attempt as u64);
                    let jitter = rand::thread_rng().gen_range(0..=150);
                    tracing::warn!(
                        "EM 板块接口请求失败，准备重试: error={}, attempt={}",
                        e,
                        attempt
                    );
                    sleep(Duration::from_millis(backoff + jitter)).await;
                    continue;
                }
                return Err(e.into());
            }
        }
    };

    let data = json.get("data").ok_or(EmPlateError::MissingData)?;
    let total = data.get("total").and_then(|v| v.as_i64()).unwrap_or(0);
    let mut items = Vec::new();
    if let Some(diff) = data.get("diff").and_then(|v| v.as_array()) {
        for item in diff {
            let plate_code = item
                .get("f12")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = item
                .get("f14")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if !plate_code.is_empty() && !name.is_empty() {
                items.push(EmPlateItem { plate_code, name });
            }
        }
    }

    Ok(EmPlateResponse { total, items })
}
