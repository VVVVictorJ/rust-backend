// @generated automatically by Diesel CLI based on the provided DDL.
diesel::table! {
    stock_requests (id) {
        id -> Int4,
        request_uuid -> Uuid,
        request_time -> Timestamptz,
        strategy_name -> Nullable<Varchar>,
        time_range_start -> Nullable<Date>,
        time_range_end -> Nullable<Date>,
    }
}

diesel::table! {
    stock_request_stocks (request_id, stock_code) {
        request_id -> Int4,
        stock_code -> Varchar,
    }
}

diesel::table! {
    stock_snapshots (id) {
        id -> Int4,
        request_id -> Int4,
        stock_code -> Varchar,
        stock_name -> Varchar,
        latest_price -> Numeric,
        change_pct -> Numeric,
        volume_ratio -> Numeric,
        turnover_rate -> Numeric,
        bid_ask_ratio -> Numeric,
        main_force_inflow -> Numeric,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    daily_klines (stock_code, trade_date) {
        stock_code -> Varchar,
        trade_date -> Date,
        open_price -> Numeric,
        high_price -> Numeric,
        low_price -> Numeric,
        close_price -> Numeric,
        volume -> Int8,
        amount -> Numeric,
    }
}

diesel::table! {
    profit_analysis (id) {
        id -> Int4,
        snapshot_id -> Int4,
        strategy_name -> Varchar,
        profit_rate -> Numeric,
        analysis_time -> Timestamptz,
    }
}

diesel::joinable!(stock_request_stocks -> stock_requests (request_id));
diesel::joinable!(stock_snapshots -> stock_requests (request_id));
diesel::joinable!(profit_analysis -> stock_snapshots (snapshot_id));

diesel::allow_tables_to_appear_in_same_query!(
    stock_requests,
    stock_request_stocks,
    stock_snapshots,
    daily_klines,
    profit_analysis,
);

