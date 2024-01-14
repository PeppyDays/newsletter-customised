use std::ops::Add;

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct SubscriptionToken {
    // TODO: Make fields private
    pub token: String,
    pub subscriber_id: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expired_at: DateTime<Utc>,
}

impl SubscriptionToken {
    pub fn issue(subscriber_id: Uuid) -> Self {
        // TODO: Separate token generation logic from this model
        // TODO: Make token expiration duration configurable
        let token = Uuid::new_v4().to_string();
        let expiration_duration = Duration::hours(1);

        Self {
            token,
            subscriber_id,
            issued_at: Utc::now(),
            expired_at: Utc::now().add(expiration_duration),
        }
    }
}
