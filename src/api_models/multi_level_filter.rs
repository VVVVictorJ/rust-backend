use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlateBrief {
    pub plate_code: String,
    pub name: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct MonthlyMaCrossRequest {
    /// 与 `anchor_month` 同时提供时，以该「年-月」对应月 K 为锚点；否则用最后一根月 K。
    #[serde(default)]
    pub anchor_year: Option<i32>,
    #[serde(default)]
    pub anchor_month: Option<u32>,
    /// 所选板块任一命中即纳入扫描；空表示不筛选（与交易日查询板块筛选语义一致）。
    #[serde(default)]
    pub filter_plate_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyMaCrossItem {
    pub stock_code: String,
    pub stock_name: String,
    pub latest_price: BigDecimal,
    pub plates: Vec<PlateBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ma5_current: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ma20_current: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ma5_prev: Option<BigDecimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ma20_prev: Option<BigDecimal>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkippedStock {
    pub stock_code: String,
    pub stock_name: String,
    pub plates: Vec<PlateBrief>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonthlyMaCrossResponse {
    pub items: Vec<MonthlyMaCrossItem>,
    pub skipped: Vec<SkippedStock>,
}

/// 日线在刚上穿：一次返回月线全量扫描结果 + 月线命中基础上的日线判别，前端两 Tab 同步更新。
#[derive(Debug, Clone, Serialize)]
pub struct DailyAfterMonthlyMaCrossResponse {
    pub monthly: MonthlyMaCrossResponse,
    pub daily_refinement: MonthlyMaCrossResponse,
}
