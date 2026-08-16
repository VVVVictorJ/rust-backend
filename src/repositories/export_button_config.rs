use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::OptionalExtension;

use crate::models::{ExportButtonConfig, NewExportButtonConfig, UpdateExportButtonConfig};
use crate::schema::export_button_config::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(
    conn: &mut PgPoolConn,
    new_item: &NewExportButtonConfig,
) -> Result<ExportButtonConfig, diesel::result::Error> {
    diesel::insert_into(export_button_config)
        .values(new_item)
        .get_result(conn)
}

pub fn find_by_id(
    conn: &mut PgPoolConn,
    item_id: i32,
) -> Result<Option<ExportButtonConfig>, diesel::result::Error> {
    export_button_config
        .filter(id.eq(item_id))
        .first::<ExportButtonConfig>(conn)
        .optional()
}

pub fn list_all(conn: &mut PgPoolConn) -> Result<Vec<ExportButtonConfig>, diesel::result::Error> {
    export_button_config
        .order(sort_order.asc())
        .then_order_by(id.asc())
        .load(conn)
}

pub fn list_by_page_key(
    conn: &mut PgPoolConn,
    key: &str,
) -> Result<Vec<ExportButtonConfig>, diesel::result::Error> {
    export_button_config
        .filter(page_key.eq(key))
        .order(sort_order.asc())
        .then_order_by(id.asc())
        .load(conn)
}

pub fn update_by_id(
    conn: &mut PgPoolConn,
    item_id: i32,
    update_data: &UpdateExportButtonConfig,
) -> Result<ExportButtonConfig, diesel::result::Error> {
    diesel::update(export_button_config.filter(id.eq(item_id)))
        .set(update_data)
        .get_result(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, item_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(export_button_config.filter(id.eq(item_id))).execute(conn)
}
