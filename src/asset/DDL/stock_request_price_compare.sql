SELECT DISTINCT 
    b.stock_code,
    b.stock_name,
    b.latest_price ,
    c.high_price ,
    c.close_price ,
    c.open_price ,
    c.low_price ,
    case 
    	when a.profit_rate = 2 then 'A'
    	when a.profit_rate = 1 then 'B'
    	else 'C'
    end as "grade",
    b.created_at 
FROM profit_analysis a 
LEFT JOIN stock_snapshots b ON a.snapshot_id = b.id 
LEFT JOIN daily_klines c ON b.stock_code = c.stock_code 
WHERE a.profit_rate IN (1, 2) 
  AND b.created_at::date = '2025-12-29'   
  AND c.trade_date = '2025-12-30';
