use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// An ID used to uniquely identify a book.
/// Determined using the current timestamp, resulting in very little risk of collisions.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize, Eq, Hash)]
pub struct ID {
    id: u128,
}

impl ID {
    /// Generates an ID using system time.
    pub(crate) fn generate() -> Self {
        let now = SystemTime::now();

        let unix_timestamp = now
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time has gone backwards.");

        Self {
            id: unix_timestamp.as_nanos(),
        }
    }
}
