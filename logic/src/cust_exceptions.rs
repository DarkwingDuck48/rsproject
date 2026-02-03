use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProjectCreationErrors {
    #[error("invalid Task periods (date_start {date_start:?} >= {date_end:?})")]
    InvalidTaskDuration {
        date_start: DateTime<Utc>,
        date_end: DateTime<Utc>,
    },
    #[error("unknown project customisation error")]
    Unknown,
}
