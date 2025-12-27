begin;

-- 股票分析请求主表
-- 记录每次用户发起的股票分析请求元信息，对外通过 request_uuid 暴露
CREATE TABLE stock_requests (
    id SERIAL PRIMARY KEY,                     -- 内部自增主键，用于高效关联
    request_uuid UUID NOT NULL DEFAULT gen_random_uuid(), -- 对外唯一标识，用于 API 返回或重放
    request_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),      -- 请求发起时间（带时区）
    strategy_name VARCHAR(50),                 -- 可选：指定使用的分析策略名称（如 "momentum_v1"）
    time_range_start DATE,                     -- 可选：分析时间范围起始日（用于回测等场景）
    time_range_end DATE,                       -- 可选：分析时间范围结束日
    UNIQUE(request_uuid)
);

-- 请求关联的股票代码列表
-- 一个请求可包含多只股票，采用一对多关系建模，避免非规范化存储
CREATE TABLE stock_request_stocks (
    request_id INT NOT NULL REFERENCES stock_requests(id) ON DELETE CASCADE,
    stock_code VARCHAR(10) NOT NULL,           -- 股票代码，如 'SH600519'
    PRIMARY KEY (request_id, stock_code)
);

-- 股票快照行情数据表
-- 存储某次请求中每只股票的实时/快照行情指标
CREATE TABLE stock_snapshots (
    id SERIAL PRIMARY KEY,                     -- 快照记录内部ID
    request_id INT NOT NULL REFERENCES stock_requests(id) ON DELETE CASCADE, -- 关联原始请求
    stock_code VARCHAR(10) NOT NULL,           -- 股票代码
    stock_name VARCHAR(50) NOT NULL,           -- 股票名称
    latest_price NUMERIC(10,2) NOT NULL,       -- 最新价格
    change_pct NUMERIC(5,2) NOT NULL,          -- 涨跌幅（%）
    volume_ratio NUMERIC(6,2) NOT NULL,        -- 量比
    turnover_rate NUMERIC(6,4) NOT NULL,       -- 换手率（%）
    bid_ask_ratio NUMERIC(5,2) NOT NULL,       -- 委买委卖比
    main_force_inflow NUMERIC(12,2) NOT NULL,  -- 主力资金净流入（万元）
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() -- 快照生成时间
);

-- 日K线行情表（时序数据）
-- 存储所有股票的历史日线数据，按股票+日期联合主键组织
CREATE TABLE daily_klines (
    stock_code VARCHAR(10) NOT NULL,           -- 股票代码
    trade_date DATE NOT NULL,                  -- 交易日期
    open_price NUMERIC(10,2) NOT NULL,         -- 开盘价
    high_price NUMERIC(10,2) NOT NULL,         -- 最高价
    low_price NUMERIC(10,2) NOT NULL,          -- 最低价
    close_price NUMERIC(10,2) NOT NULL,        -- 收盘价
    volume BIGINT NOT NULL,                    -- 成交量（股）
    amount NUMERIC(15,2) NOT NULL,             -- 成交额（元）
    PRIMARY KEY (stock_code, trade_date)
);

-- 盈利分析结果表
-- 存储基于某快照执行特定策略后的收益分析结果
CREATE TABLE profit_analysis (
    id SERIAL PRIMARY KEY,                     -- 分析结果ID
    snapshot_id INT NOT NULL REFERENCES stock_snapshots(id) ON DELETE CASCADE, -- 关联快照
    strategy_name VARCHAR(50) NOT NULL,         -- 使用的策略名称（与请求中可不同，支持多策略）
    profit_rate NUMERIC(5,2) NOT NULL,         -- 预期收益率（%）
    analysis_time TIMESTAMPTZ NOT NULL DEFAULT NOW() -- 分析完成时间
);

commit;
