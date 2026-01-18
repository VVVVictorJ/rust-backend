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
    stock_plate (id) {
        id -> Int4,
        plate_code -> Varchar,
        name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stock_table (id) {
        id -> Int4,
        stock_code -> Varchar,
        stock_name -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stock_plate_stock_table (plate_id, stock_table_id) {
        plate_id -> Int4,
        stock_table_id -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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

diesel::table! {
    job_execution_history (id) {
        id -> Int4,
        job_name -> Varchar,
        status -> Varchar,
        started_at -> Timestamp,
        completed_at -> Nullable<Timestamp>,
        total_count -> Int4,
        success_count -> Int4,
        failed_count -> Int4,
        skipped_count -> Int4,
        details -> Nullable<Jsonb>,
        error_message -> Nullable<Text>,
        duration_ms -> Nullable<Int8>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(stock_request_stocks -> stock_requests (request_id));
diesel::joinable!(stock_snapshots -> stock_requests (request_id));
diesel::joinable!(profit_analysis -> stock_snapshots (snapshot_id));
diesel::joinable!(stock_plate_stock_table -> stock_plate (plate_id));
diesel::joinable!(stock_plate_stock_table -> stock_table (stock_table_id));

diesel::allow_tables_to_appear_in_same_query!(
    stock_requests,
    stock_request_stocks,
    stock_plate,
    stock_table,
    stock_plate_stock_table,
    stock_snapshots,
    daily_klines,
    profit_analysis,
    job_execution_history,
);

