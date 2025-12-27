use bigdecimal::BigDecimal;
use serde_json::Value;
use std::str::FromStr;

/// 将 JSON Value 解析为 BigDecimal
/// 
/// 支持的输入类型：
/// - Number: 转换为 f64 后再转为 BigDecimal
/// - String: 直接解析字符串为 BigDecimal
/// - 其他: 返回 BigDecimal(0)
pub fn parse_bigdecimal(v: Option<&Value>) -> BigDecimal {
    match v {
        Some(Value::Number(n)) => {
            if let Some(f) = n.as_f64() {
                BigDecimal::from_str(&f.to_string()).unwrap_or_else(|_| BigDecimal::from(0))
            } else {
                BigDecimal::from(0)
            }
        }
        Some(Value::String(s)) => BigDecimal::from_str(s).unwrap_or_else(|_| BigDecimal::from(0)),
        _ => BigDecimal::from(0),
    }
}

