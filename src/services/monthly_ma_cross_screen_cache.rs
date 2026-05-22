//! 「多级筛选 / 月线 MA 金叉」同一天内、`stock_snapshots` 快照集合指纹不变时隔日重复请求可走内存缓存，
//! 避免对东财月线接口的重复 proxy 批量拉取。

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{NaiveDate, Utc};
use chrono_tz::Asia::Shanghai;
use tokio::sync::RwLock;

use crate::api_models::multi_level_filter::{MonthlyMaCrossRequest, MonthlyMaCrossResponse};
use crate::repositories::stock_snapshot::LatestSnapshotFields;

/// 当前「上海日历日」（与 `stock_snapshots` / 交易日的日界思路一致）。
pub fn shanghai_calendar_date_now() -> NaiveDate {
    Utc::now().with_timezone(&Shanghai).date_naive()
}

/// 同一天内判断是否「列表未变化」：**已按筛选条件截断后的**快照股的 `stock_code` 集合 + 请求参数指纹。
pub fn ma_cross_screen_fingerprint(
    req: &MonthlyMaCrossRequest,
    snapshots: &[LatestSnapshotFields],
) -> String {
    let mut codes: Vec<&str> = snapshots.iter().map(|s| s.stock_code.as_str()).collect();
    codes.sort_unstable();
    codes.dedup();
    let mut plates_f = req.filter_plate_codes.clone();
    plates_f.sort();
    plates_f.dedup();

    format!(
        "v1|c:{}|pf:{}|ay:{}|am:{}",
        codes.join(","),
        plates_f.join(","),
        req.anchor_year.map(|x| x.to_string()).unwrap_or_default(),
        req.anchor_month.map(|x| x.to_string()).unwrap_or_default(),
    )
}

/// 占位日：首次写入时会按真实上海日 rollover。
fn sentinel_calendar_day() -> NaiveDate {
    NaiveDate::from_ymd_opt(1900, 1, 1).expect("sentinel date")
}

struct CacheInner {
    calendar_day_sh: NaiveDate,
    entries: HashMap<String, MonthlyMaCrossResponse>,
}

impl Default for CacheInner {
    fn default() -> Self {
        Self {
            calendar_day_sh: sentinel_calendar_day(),
            entries: HashMap::new(),
        }
    }
}

impl CacheInner {
    fn rollover_if_new_day(&mut self, today: NaiveDate) {
        if self.calendar_day_sh != today {
            self.entries.clear();
            self.calendar_day_sh = today;
        }
    }
}

#[derive(Clone)]
pub struct MaCrossScreenCache {
    inner: Arc<RwLock<CacheInner>>,
}

impl Default for MaCrossScreenCache {
    fn default() -> Self {
        Self {
            inner: Arc::new(RwLock::new(CacheInner::default())),
        }
    }
}

impl MaCrossScreenCache {
    pub async fn try_hit(
        &self,
        today: NaiveDate,
        fingerprint: &str,
    ) -> Option<MonthlyMaCrossResponse> {
        let guard = self.inner.read().await;
        if guard.calendar_day_sh != today {
            return None;
        }
        guard.entries.get(fingerprint).cloned()
    }

    pub async fn insert(
        &self,
        today: NaiveDate,
        fingerprint: String,
        response: MonthlyMaCrossResponse,
    ) {
        let mut w = self.inner.write().await;
        w.rollover_if_new_day(today);
        w.entries.insert(fingerprint, response);
    }
}
