use std::ops::Add;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct SubscriptionToken {
    pub token: String,
    pub subscriber_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

impl SubscriptionToken {
    const EXPIRATION_DURATION_IN_HOURS: i64 = 1;

    pub fn issue(subscriber_id: Uuid) -> Self {
        let token = Uuid::new_v4().to_string();
        let expiration_duration = Duration::hours(Self::EXPIRATION_DURATION_IN_HOURS);

        Self {
            token,
            subscriber_id,
            issued_at: Utc::now(),
            expired_at: Utc::now().add(expiration_duration),
        }
    }
}
