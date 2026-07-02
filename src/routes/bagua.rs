use axum::{
    routing::get,
    Router,
};

use crate::app::AppState;
use crate::handler::bagua::{
    get_almanac, get_hetu_cross_lookup, get_hetu_lookup, get_luoshu_branch_lookup,
    get_luoshu_stem_lookup,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/lookup/hetu", get(get_hetu_lookup))
        .route("/lookup/hetu-cross", get(get_hetu_cross_lookup))
        .route("/lookup/luoshu-stem", get(get_luoshu_stem_lookup))
        .route("/lookup/luoshu-branch", get(get_luoshu_branch_lookup))
        .route("/almanac", get(get_almanac))
}
