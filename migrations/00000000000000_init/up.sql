-- Enable pgcrypto for gen_random_uuid (safe if exists)
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- stock_requests
CREATE TABLE IF NOT EXISTS stock_requests (
    id SERIAL PRIMARY KEY,
    request_uuid UUID NOT NULL DEFAULT gen_random_uuid(),
    request_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    strategy_name VARCHAR(50),
    time_range_start DATE,
    time_range_end DATE,
    UNIQUE(request_uuid)
);

-- stock_request_stocks
CREATE TABLE IF NOT EXISTS stock_request_stocks (
    request_id INT NOT NULL REFERENCES stock_requests(id) ON DELETE CASCADE,
    stock_code VARCHAR(10) NOT NULL,
    PRIMARY KEY (request_id, stock_code)
);

-- stock_snapshots
CREATE TABLE IF NOT EXISTS stock_snapshots (
    id SERIAL PRIMARY KEY,
    request_id INT NOT NULL REFERENCES stock_requests(id) ON DELETE CASCADE,
    stock_code VARCHAR(10) NOT NULL,
    stock_name VARCHAR(50) NOT NULL,
    latest_price NUMERIC(10,2) NOT NULL,
    change_pct NUMERIC(5,2) NOT NULL,
    volume_ratio NUMERIC(6,2) NOT NULL,
    turnover_rate NUMERIC(6,4) NOT NULL,
    bid_ask_ratio NUMERIC(5,2) NOT NULL,
    main_force_inflow NUMERIC(12,2) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- daily_klines
CREATE TABLE IF NOT EXISTS daily_klines (
    stock_code VARCHAR(10) NOT NULL,
    trade_date DATE NOT NULL,
    open_price NUMERIC(10,2) NOT NULL,
    high_price NUMERIC(10,2) NOT NULL,
    low_price NUMERIC(10,2) NOT NULL,
    close_price NUMERIC(10,2) NOT NULL,
    volume BIGINT NOT NULL,
    amount NUMERIC(15,2) NOT NULL,
    PRIMARY KEY (stock_code, trade_date)
);

-- profit_analysis
CREATE TABLE IF NOT EXISTS profit_analysis (
    id SERIAL PRIMARY KEY,
    snapshot_id INT NOT NULL REFERENCES stock_snapshots(id) ON DELETE CASCADE,
    strategy_name VARCHAR(50) NOT NULL,
    profit_rate NUMERIC(5,2) NOT NULL,
    analysis_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

