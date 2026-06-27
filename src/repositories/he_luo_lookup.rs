use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::HeLuoLookup;
use crate::schema::he_luo_lookup::dsl::{col_key, he_luo_lookup, matrix_code, row_key};

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn list_by_matrix(
    conn: &mut PgPoolConn,
    code: &str,
) -> Result<Vec<HeLuoLookup>, diesel::result::Error> {
    he_luo_lookup
        .filter(matrix_code.eq(code))
        .order((row_key.asc(), col_key.asc()))
        .load(conn)
}

pub fn find_cell(
    conn: &mut PgPoolConn,
    code: &str,
    row: &str,
    col: &str,
) -> Result<Option<HeLuoLookup>, diesel::result::Error> {
    he_luo_lookup
        .filter(matrix_code.eq(code))
        .filter(row_key.eq(row))
        .filter(col_key.eq(col))
        .first::<HeLuoLookup>(conn)
        .optional()
}
