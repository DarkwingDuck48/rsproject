use chrono::{TimeZone, Utc};
use logic::Project;

fn main() {
    let date_start = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();
    let date_end = Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap();

    let project = Project::new("TestProject", "Some test project", date_start, date_end);
    println!("{}", project);
}
