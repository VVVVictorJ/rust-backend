use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, PooledConnection};

use crate::models::{NewProfitAnalysis, ProfitAnalysis};
use crate::schema::profit_analysis::dsl::*;

pub type PgPoolConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn find_by_id(conn: &mut PgPoolConn, analysis_id: i32) -> Result<ProfitAnalysis, diesel::result::Error> {
    profit_analysis.find(analysis_id).first(conn)
}

pub fn delete_by_id(conn: &mut PgPoolConn, analysis_id: i32) -> Result<usize, diesel::result::Error> {
    diesel::delete(profit_analysis.find(analysis_id)).execute(conn)
}

pub fn create(conn: &mut PgPoolConn, new_rec: &NewProfitAnalysis) -> Result<i32, diesel::result::Error> {
    diesel::insert_into(profit_analysis)
        .values(new_rec)
        .returning(id)
        .get_result(conn)
}

