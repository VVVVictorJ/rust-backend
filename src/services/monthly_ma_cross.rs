//! 月线 MA5 / MA20 「刚上穿」判定（上一根月 K：MA5≤MA20；当前锚定月 K：MA5>MA20）。

use std::cmp::Ordering;

use bigdecimal::BigDecimal;
use chrono::Datelike;

use crate::models::NewDailyKline;

#[derive(Debug, Clone)]
pub struct MonthlyCrossMetrics {
    pub ma5_current: BigDecimal,
    pub ma20_current: BigDecimal,
    pub ma5_prev: BigDecimal,
    pub ma20_prev: BigDecimal,
}

#[derive(Debug)]
pub enum EvalOutcome {
    Hit(MonthlyCrossMetrics),
    Miss { reason: String },
}

fn avg(closes: &[BigDecimal]) -> BigDecimal {
    let sum = closes.iter().fold(BigDecimal::from(0u8), |acc, x| acc + x);
    sum / BigDecimal::from(closes.len() as i64)
}

/// `anchor_year` / `anchor_month` 须同时提供或同时省略；省略时使用最后一根月 K。
pub fn eval_monthly_ma5_cross_ma20(
    mut klines: Vec<NewDailyKline>,
    anchor_year: Option<i32>,
    anchor_month: Option<u32>,
) -> Result<EvalOutcome, String> {
    match (anchor_year, anchor_month) {
        (None, None) => {}
        (Some(_), Some(_)) => {}
        _ => {
            return Err(
                "anchor_year and anchor_month must both be set or both omitted".to_string(),
            );
        }
    }

    klines.sort_by(|a, b| a.trade_date.cmp(&b.trade_date));
    let n = klines.len();
    if n < 21 {
        return Ok(EvalOutcome::Miss {
            reason: "insufficient monthly bars (need >= 21)".to_string(),
        });
    }

    let i = if let (Some(y), Some(m)) = (anchor_year, anchor_month) {
        if !(1..=12).contains(&m) {
            return Err("anchor_month must be in 1..=12".to_string());
        }
        let mut idx_opt = None;
        for (idx, k) in klines.iter().enumerate() {
            if k.trade_date.year() == y && k.trade_date.month() == m {
                idx_opt = Some(idx);
            }
        }
        match idx_opt {
            Some(idx) => idx,
            None => {
                return Ok(EvalOutcome::Miss {
                    reason: format!("no monthly bar for anchor {y}-{m:02}"),
                });
            }
        }
    } else {
        n - 1
    };

    if i < 20 {
        return Ok(EvalOutcome::Miss {
            reason: "anchor index too early for MA20 window (need i >= 20)".to_string(),
        });
    }

    let closes: Vec<BigDecimal> = klines.iter().map(|k| k.close_price.clone()).collect();

    let ma5_curr = avg(&closes[i - 4..=i]);
    let ma20_curr = avg(&closes[i - 19..=i]);
    let ma5_prev = avg(&closes[i - 5..=i - 1]);
    let ma20_prev = avg(&closes[i - 20..=i - 1]);

    let prev_below_or_equal = matches!(ma5_prev.cmp(&ma20_prev), Ordering::Less | Ordering::Equal);

    if prev_below_or_equal && ma5_curr > ma20_curr {
        return Ok(EvalOutcome::Hit(MonthlyCrossMetrics {
            ma5_current: ma5_curr,
            ma20_current: ma20_curr,
            ma5_prev,
            ma20_prev,
        }));
    }

    Ok(EvalOutcome::Miss {
        reason: "MA5/MA20 cross condition not met on anchor bar".to_string(),
    })
}
