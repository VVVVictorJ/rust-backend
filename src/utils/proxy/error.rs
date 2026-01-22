use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("缺少环境变量 {0}")]
    MissingEnv(&'static str),
    #[error("代理 API 返回错误码 {code}: {message} (request_id={request_id:?})")]
    Api {
        code: String,
        message: String,
        request_id: Option<String>,
    },
    #[error("HTTP 请求失败: {0}")]
    Http(#[from] reqwest::Error),
    #[error("HTTP 状态错误 {status}: {body}")]
    Status { status: StatusCode, body: String },
    #[error("解析失败: {0}")]
    Parse(String),
    #[error("代理数据为空")]
    NoProxyData,
    #[error("代理地址无效: {0}")]
    InvalidProxyUrl(String),
}

pub(crate) fn map_error_message(code: &str) -> &'static str {
    match code {
        "INTERNAL_ERROR" => "系统内部异常。",
        "INVALID_PARAMETER" => "参数错误（包含参数格式、类型等错误）。",
        "INVALID_KEY" => "Key不存在或已过期。",
        "UNAVAILABLE_KEY" => "Key不可用，已过期或被封禁。",
        "ACCESS_DENY" => "Key没有此接口的权限。",
        "API_AUTH_DENY" => "Api授权不通过，请检查API鉴权配置。",
        "KEY_BLOCK" => "Key被封禁。",
        "REQUEST_LIMIT_EXCEEDED" => "请求频率超出限制。",
        "BALANCE_INSUFFICIENT" => "Key余额不足。",
        "NO_RESOURCE_FOUND" => "资源不足。",
        "FAILED_OPERATION" => "提取失败。",
        "EXTRACT_LIMIT_EXCEEDED" => "超出提取配额。",
        _ => "未知错误码。",
    }
}
