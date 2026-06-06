use chrono::Local;

pub fn resolve_datetime(format: Option<&str>, fallback: &str) -> String {
    Local::now().format(format.unwrap_or(fallback)).to_string()
}
