SELECT 
    a.stock_code,
    a.stock_name,
    a.latest_price,
    dk.close_price,
    a.change_pct,
    a.volume_ratio,
    a.turnover_rate,
    a.bid_ask_ratio,
    a.main_force_inflow,
    a.created_at
FROM stock_snapshots a 
LEFT JOIN daily_klines dk ON a.stock_code = dk.stock_code 
WHERE a.main_force_inflow > 0
  AND (a.created_at AT TIME ZONE 'Asia/Shanghai')::date = dk.trade_date 
  AND dk.trade_date = '2025-12-29'
