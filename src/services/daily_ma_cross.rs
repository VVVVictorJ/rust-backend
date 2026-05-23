//! 日线 MA5 / MA20 「刚上穿」：锚定**最后一根日 K**（等价于不传 anchor 年月时的月线判定）。
//!
//! 实现直接复用 [`crate::services::monthly_ma_cross::eval_monthly_ma5_cross_ma20`]（`anchor_year/month` 为 `None` 时即以序列末根 K 为「当前」bar）。

use crate::models::NewDailyKline;

pub fn eval_daily_ma5_cross_ma20(
    klines: Vec<NewDailyKline>,
) -> Result<crate::services::monthly_ma_cross::EvalOutcome, String> {
    crate::services::monthly_ma_cross::eval_monthly_ma5_cross_ma20(klines, None, None)
}
