use axum::{
    routing::{get, post},
    Router,
};

use crate::app::AppState;
use crate::handler::export_button_config::{
    create_export_button_config, delete_export_button_config, get_export_button_config,
    list_export_button_configs, update_export_button_config,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_export_button_config).get(list_export_button_configs))
        .route(
            "/:id",
            get(get_export_button_config)
                .put(update_export_button_config)
                .delete(delete_export_button_config),
        )
}
