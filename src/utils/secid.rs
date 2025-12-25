pub fn code_to_secid(code: &str) -> String {
    let trimmed = code.trim();
    if trimmed.starts_with('6') {
        format!("1.{}", trimmed)
    } else {
        format!("0.{}", trimmed)
    }
}
