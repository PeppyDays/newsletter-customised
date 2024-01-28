use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::{Subscriber, SubscriberStatus};

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_status(
        &self,
        status: SubscriberStatus,
    ) -> Result<Vec<Subscriber>, SubscriberError>;
}