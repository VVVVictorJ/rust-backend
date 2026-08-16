use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;

use crate::api_models::export_button_config::{
    CreateExportButtonConfigRequest, ExportButtonConfigResponse, UpdateExportButtonConfigRequest,
};
use crate::app::AppState;
use crate::handler::error::AppError;
use crate::models::{NewExportButtonConfig, UpdateExportButtonConfig};
use crate::repositories::export_button_config;

#[derive(Debug, Deserialize)]
pub struct ListQueryParams {
    pub page_key: Option<String>,
}

/// 创建导出按钮配置
pub async fn create_export_button_config(
    State(state): State<AppState>,
    Json(payload): Json<CreateExportButtonConfigRequest>,
) -> Result<(StatusCode, Json<ExportButtonConfigResponse>), AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let new_item: NewExportButtonConfig = payload.into();
    let created = export_button_config::create(&mut conn, &new_item).map_err(|e| {
        tracing::error!("Failed to create export button config: {}", e);
        AppError::InternalServerError
    })?;

    Ok((StatusCode::CREATED, Json(created.into())))
}

/// 获取导出按钮配置列表（支持按 page_key 过滤）
pub async fn list_export_button_configs(
    State(state): State<AppState>,
    Query(params): Query<ListQueryParams>,
) -> Result<Json<Vec<ExportButtonConfigResponse>>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let items = if let Some(key) = params.page_key {
        export_button_config::list_by_page_key(&mut conn, &key).map_err(|e| {
            tracing::error!("Failed to list export button configs by page_key: {}", e);
            AppError::InternalServerError
        })?
    } else {
        export_button_config::list_all(&mut conn).map_err(|e| {
            tracing::error!("Failed to list export button configs: {}", e);
            AppError::InternalServerError
        })?
    };

    let response: Vec<ExportButtonConfigResponse> = items.into_iter().map(Into::into).collect();
    Ok(Json(response))
}

/// 获取单个导出按钮配置
pub async fn get_export_button_config(
    State(state): State<AppState>,
    Path(item_id): Path<i32>,
) -> Result<Json<ExportButtonConfigResponse>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let item = export_button_config::find_by_id(&mut conn, item_id)
        .map_err(|e| {
            tracing::error!("Failed to find export button config: {}", e);
            AppError::InternalServerError
        })?
        .ok_or(AppError::NotFound)?;

    Ok(Json(item.into()))
}

/// 更新导出按钮配置
pub async fn update_export_button_config(
    State(state): State<AppState>,
    Path(item_id): Path<i32>,
    Json(payload): Json<UpdateExportButtonConfigRequest>,
) -> Result<Json<ExportButtonConfigResponse>, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let update_data = UpdateExportButtonConfig {
        page_key: payload.page_key,
        name: payload.name,
        plate_codes: payload.plate_codes.map(serde_json::Value::from),
        sort_order: payload.sort_order,
        updated_at: Some(chrono::Utc::now().naive_utc()),
    };

    let updated = export_button_config::update_by_id(&mut conn, item_id, &update_data).map_err(|e| {
        tracing::error!("Failed to update export button config: {}", e);
        AppError::InternalServerError
    })?;

    Ok(Json(updated.into()))
}

/// 删除导出按钮配置
pub async fn delete_export_button_config(
    State(state): State<AppState>,
    Path(item_id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let mut conn = state
        .db_pool
        .get()
        .map_err(|_| AppError::InternalServerError)?;

    let affected = export_button_config::delete_by_id(&mut conn, item_id).map_err(|e| {
        tracing::error!("Failed to delete export button config: {}", e);
        AppError::InternalServerError
    })?;

    if affected == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
