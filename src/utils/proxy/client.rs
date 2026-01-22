use std::env;
use std::time::Duration as StdDuration;

use chrono::{DateTime, Duration, Local, NaiveDateTime, TimeZone};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Proxy};
use serde::Deserialize;

use super::error::{map_error_message, ProxyError};

const DEFAULT_API_URL: &str =
    "https://share.proxy.qg.net/get?key=JRAYCT2X&num=1&area=310000&distinct=true";
const DEFAULT_TTL_SECS: i64 = 60;
const DEFAULT_MAX_RETRIES: usize = 3;
const DEFAULT_TIMEOUT_SECS: u64 = 15;

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub api_url: String,
    pub auth_key: String,
    pub auth_pwd: String,
    pub max_retries: usize,
    pub timeout_secs: u64,
}

impl ProxyConfig {
    pub fn from_env() -> Result<Self, ProxyError> {
        let api_url = env::var("PROXY_API_URL").unwrap_or_else(|_| DEFAULT_API_URL.to_string());
        let auth_key =
            env::var("PROXY_AUTH_KEY").map_err(|_| ProxyError::MissingEnv("PROXY_AUTH_KEY"))?;
        let auth_pwd =
            env::var("PROXY_AUTH_PWD").map_err(|_| ProxyError::MissingEnv("PROXY_AUTH_PWD"))?;
        let max_retries = env::var("PROXY_MAX_RETRIES")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_RETRIES);
        let timeout_secs = env::var("PROXY_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        Ok(Self {
            api_url,
            auth_key,
            auth_pwd,
            max_retries,
            timeout_secs,
        })
    }
}

#[derive(Debug, Clone)]
struct CachedProxy {
    client: Client,
    proxy_ip: String,
    server: String,
    deadline: DateTime<Local>,
}

impl CachedProxy {
    fn is_expired(&self) -> bool {
        Local::now() >= self.deadline
    }
}

#[derive(Debug)]
pub struct ProxyClient {
    api_url: String,
    auth_key: String,
    auth_pwd: String,
    max_retries: usize,
    timeout: StdDuration,
    cached: Option<CachedProxy>,
}

impl ProxyClient {
    pub fn new(config: ProxyConfig) -> Self {
        Self {
            api_url: config.api_url,
            auth_key: config.auth_key,
            auth_pwd: config.auth_pwd,
            max_retries: config.max_retries,
            timeout: StdDuration::from_secs(config.timeout_secs),
            cached: None,
        }
    }

    pub fn from_env() -> Result<Self, ProxyError> {
        Ok(Self::new(ProxyConfig::from_env()?))
    }

    pub fn invalidate_proxy(&mut self) {
        self.cached = None;
    }

    pub async fn get_with_proxy(&mut self, url: &str) -> Result<String, ProxyError> {
        self.request_with_proxy(Method::GET, url, None, None)
            .await
    }

    pub async fn request_with_proxy(
        &mut self,
        method: Method,
        url: &str,
        headers: Option<HeaderMap>,
        body: Option<Vec<u8>>,
    ) -> Result<String, ProxyError> {
        let mut last_error = None;

        for _ in 0..self.max_retries {
            let client = self.ensure_proxy_client().await?;
            let mut request = client.request(method.clone(), url);

            if let Some(ref headers) = headers {
                request = request.headers(headers.clone());
            }

            if let Some(ref body) = body {
                request = request.body(body.clone());
            }

            match request.send().await {
                Ok(resp) => {
                    let status = resp.status();
                    let text = resp.text().await.unwrap_or_default();
                    if status.is_success() {
                        return Ok(text);
                    }

                    self.invalidate_proxy();
                    last_error = Some(ProxyError::Status { status, body: text });
                }
                Err(err) => {
                    self.invalidate_proxy();
                    last_error = Some(ProxyError::Http(err));
                }
            }
        }

        Err(last_error.unwrap_or(ProxyError::NoProxyData))
    }

    async fn ensure_proxy_client(&mut self) -> Result<Client, ProxyError> {
        if let Some(cached) = &self.cached {
            if !cached.is_expired() {
                return Ok(cached.client.clone());
            }
        }

        let proxy_entry = self.fetch_proxy().await?;
        let client = self.build_proxy_client(&proxy_entry.server)?;
        self.cached = Some(CachedProxy {
            client: client.clone(),
            proxy_ip: proxy_entry.proxy_ip,
            server: proxy_entry.server,
            deadline: proxy_entry.deadline,
        });
        Ok(client)
    }

    async fn fetch_proxy(&self) -> Result<ProxyEntry, ProxyError> {
        let client = Client::builder().timeout(self.timeout).build()?;
        let resp = client.get(&self.api_url).send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            return Err(ProxyError::Status { status, body: text });
        }

        let api_resp: ProxyApiResponse = serde_json::from_str(&text)
            .map_err(|err| ProxyError::Parse(format!("解析代理响应失败: {err}")))?;

        if api_resp.code != "SUCCESS" {
            let code = api_resp.code;
            return Err(ProxyError::Api {
                code: code.clone(),
                message: map_error_message(&code).to_string(),
                request_id: api_resp.request_id,
            });
        }

        let entry = api_resp
            .data
            .into_iter()
            .next()
            .ok_or(ProxyError::NoProxyData)?;

        Ok(ProxyEntry::from_raw(entry))
    }

    fn build_proxy_client(&self, server: &str) -> Result<Client, ProxyError> {
        let proxy_url = format!("http://{}:{}@{}", self.auth_key, self.auth_pwd, server);
        let proxy = Proxy::http(&proxy_url)
            .map_err(|_| ProxyError::InvalidProxyUrl(proxy_url.clone()))?;

        Ok(Client::builder()
            .proxy(proxy)
            .timeout(self.timeout)
            .build()?)
    }
}

#[derive(Debug, Deserialize)]
struct ProxyApiResponse {
    code: String,
    data: Vec<ProxyEntryRaw>,
    request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProxyEntryRaw {
    proxy_ip: String,
    server: String,
    #[allow(dead_code)]
    area_code: Option<i64>,
    #[allow(dead_code)]
    area: Option<String>,
    #[allow(dead_code)]
    isp: Option<String>,
    deadline: String,
}

#[derive(Debug)]
struct ProxyEntry {
    proxy_ip: String,
    server: String,
    deadline: DateTime<Local>,
}

impl ProxyEntry {
    fn from_raw(raw: ProxyEntryRaw) -> Self {
        Self {
            proxy_ip: raw.proxy_ip,
            server: raw.server,
            deadline: parse_deadline(&raw.deadline),
        }
    }
}

fn parse_deadline(deadline: &str) -> DateTime<Local> {
    if let Ok(parsed) = NaiveDateTime::parse_from_str(deadline, "%Y-%m-%d %H:%M:%S") {
        if let Some(local_time) = Local.from_local_datetime(&parsed).single() {
            return local_time;
        }
    }

    Local::now() + Duration::seconds(DEFAULT_TTL_SECS)
}
