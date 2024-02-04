use std::future::Future;

use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::Subscriber;

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    async fn modify<F, Fut>(&self, id: Uuid, modifier: F) -> Result<(), SubscriberError>
    where
        F: Fn(Subscriber) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Subscriber, SubscriberError>> + Send + 'static;
    async fn find_all(&self) -> Result<Vec<Subscriber>, SubscriberError>;
}
