use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::sql_types::{BigInt, Int4, Text};

use crate::models::NewStockPlateStockTable;
use crate::schema::stock_plate_stock_table::dsl::{
    plate_id as plate_id_col, stock_plate_stock_table, stock_table_id as stock_table_id_col,
};

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn create(
    conn: &mut PgPoolConn,
    new_rel: &NewStockPlateStockTable,
) -> Result<(), diesel::result::Error> {
    diesel::insert_into(stock_plate_stock_table)
        .values(new_rel)
        .execute(conn)?;
    Ok(())
}

pub fn delete_by_pk(
    conn: &mut PgPoolConn,
    plate_id_val: i32,
    stock_table_id_val: i32,
) -> Result<usize, diesel::result::Error> {
    diesel::delete(
        stock_plate_stock_table
            .filter(plate_id_col.eq(plate_id_val))
            .filter(stock_table_id_col.eq(stock_table_id_val)),
    )
    .execute(conn)
}

#[derive(Debug, QueryableByName)]
pub struct StockPlateStockQueryResult {
    #[diesel(sql_type = Int4)]
    pub plate_id: i32,
    #[diesel(sql_type = Text)]
    pub plate_name: String,
    #[diesel(sql_type = Int4)]
    pub stock_table_id: i32,
    #[diesel(sql_type = Text)]
    pub stock_code: String,
    #[diesel(sql_type = Text)]
    pub stock_name: String,
}

pub fn query_plate_stocks(
    conn: &mut PgPoolConn,
    plate_name_filter: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<Vec<StockPlateStockQueryResult>, diesel::result::Error> {
    if let Some(name_filter) = plate_name_filter {
        let pattern = format!("%{}%", name_filter);
        let query = r#"
            SELECT
                p.id AS plate_id,
                p.name AS plate_name,
                s.id AS stock_table_id,
                s.stock_code AS stock_code,
                s.stock_name AS stock_name
            FROM stock_plate_stock_table rel
            JOIN stock_plate p ON rel.plate_id = p.id
            JOIN stock_table s ON rel.stock_table_id = s.id
            WHERE p.name ILIKE $1
            ORDER BY p.id, s.id
            LIMIT $2 OFFSET $3
        "#;

        diesel::sql_query(query)
            .bind::<Text, _>(pattern)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load::<StockPlateStockQueryResult>(conn)
    } else {
        let query = r#"
            SELECT
                p.id AS plate_id,
                p.name AS plate_name,
                s.id AS stock_table_id,
                s.stock_code AS stock_code,
                s.stock_name AS stock_name
            FROM stock_plate_stock_table rel
            JOIN stock_plate p ON rel.plate_id = p.id
            JOIN stock_table s ON rel.stock_table_id = s.id
            ORDER BY p.id, s.id
            LIMIT $1 OFFSET $2
        "#;

        diesel::sql_query(query)
            .bind::<BigInt, _>(limit)
            .bind::<BigInt, _>(offset)
            .load::<StockPlateStockQueryResult>(conn)
    }
}

pub fn count_plate_stocks(
    conn: &mut PgPoolConn,
    plate_name_filter: Option<&str>,
) -> Result<i64, diesel::result::Error> {
    #[derive(QueryableByName)]
    struct CountResult {
        #[diesel(sql_type = BigInt)]
        count: i64,
    }

    if let Some(name_filter) = plate_name_filter {
        let pattern = format!("%{}%", name_filter);
        let query = r#"
            SELECT COUNT(*) AS count
            FROM stock_plate_stock_table rel
            JOIN stock_plate p ON rel.plate_id = p.id
            JOIN stock_table s ON rel.stock_table_id = s.id
            WHERE p.name ILIKE $1
        "#;

        let result = diesel::sql_query(query)
            .bind::<Text, _>(pattern)
            .get_result::<CountResult>(conn)?;
        Ok(result.count)
    } else {
        let query = r#"
            SELECT COUNT(*) AS count
            FROM stock_plate_stock_table rel
            JOIN stock_plate p ON rel.plate_id = p.id
            JOIN stock_table s ON rel.stock_table_id = s.id
        "#;

        let result = diesel::sql_query(query)
            .get_result::<CountResult>(conn)?;
        Ok(result.count)
    }
}
