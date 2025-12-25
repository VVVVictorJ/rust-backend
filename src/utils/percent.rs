pub fn normalize_percent_scalar<S: AsRef<str>>(s: S) -> Option<f64> {
    let raw = s.as_ref().trim();
    if raw.is_empty() {
        return None;
    }
    let cleaned = raw.trim_end_matches('%').trim();
    let mut val = cleaned.parse::<f64>().ok()?;
    if val.abs() > 100.0 {
        val /= 100.0;
    }
    Some(val)
}


