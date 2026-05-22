/// 去掉可选市场前缀 / 后缀，得到东财可用的 6 位（或深市短码）字符。
pub fn normalize_stock_code_digits(code: &str) -> String {
    let t = code.trim().to_ascii_uppercase();
    if let Some(rest) = t.strip_prefix("SH") {
        return rest.to_string();
    }
    if let Some(rest) = t.strip_prefix("SZ") {
        return rest.to_string();
    }
    if let Some(rest) = t.strip_suffix(".SH") {
        return rest.to_string();
    }
    if let Some(rest) = t.strip_suffix(".SZ") {
        return rest.to_string();
    }
    t
}

/// 上交所 `secid`: `1.600519`，深交所 `secid`: `0.002816`。
pub fn code_to_secid(code: &str) -> String {
    let trimmed = normalize_stock_code_digits(code);
    if trimmed.starts_with('6') {
        format!("1.{trimmed}")
    } else {
        format!("0.{trimmed}")
    }
}
