use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 创建导出按钮配置请求
#[derive(Debug, Deserialize)]
pub struct CreateExportButtonConfigRequest {
    pub page_key: String,
    pub name: String,
    pub plate_codes: Vec<String>,
    pub sort_order: Option<i32>,
}

/// 更新导出按钮配置请求
#[derive(Debug, Deserialize, Default)]
pub struct UpdateExportButtonConfigRequest {
    pub page_key: Option<String>,
    pub name: Option<String>,
    pub plate_codes: Option<Vec<String>>,
    pub sort_order: Option<i32>,
}

/// 导出按钮配置响应
#[derive(Debug, Serialize)]
pub struct ExportButtonConfigResponse {
    pub id: i32,
    pub page_key: String,
    pub name: String,
    pub plate_codes: Vec<String>,
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<crate::models::ExportButtonConfig> for ExportButtonConfigResponse {
    fn from(item: crate::models::ExportButtonConfig) -> Self {
        let plate_codes: Vec<String> = serde_json::from_value(item.plate_codes).unwrap_or_default();
        Self {
            id: item.id,
            page_key: item.page_key,
            name: item.name,
            plate_codes,
            sort_order: item.sort_order,
            created_at: item.created_at,
            updated_at: item.updated_at,
        }
    }
}

impl From<CreateExportButtonConfigRequest> for crate::models::NewExportButtonConfig {
    fn from(req: CreateExportButtonConfigRequest) -> Self {
        Self {
            page_key: req.page_key,
            name: req.name,
            plate_codes: Value::from(req.plate_codes),
            sort_order: req.sort_order,
        }
    }
}
