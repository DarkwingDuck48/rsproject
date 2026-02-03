use chrono::{TimeZone, Utc};
use logic::Project;

fn main() {
    let start_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let end_date = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();
    let _project = Project::new("Test", "Some Test Project", start_date, end_date);
}
