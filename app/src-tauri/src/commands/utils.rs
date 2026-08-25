use chrono::{DateTime, NaiveDate, Utc};

pub fn parse_date(date: &str) -> Result<DateTime<Utc>, String> {
    let parsed_date = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .map_err(|e| e.to_string())?
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
    Ok(parsed_date)
}
