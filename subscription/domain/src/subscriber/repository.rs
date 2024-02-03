use std::future::Future;

use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::{
    Subscriber,
    SubscriberStatus,
};

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    // TODO: Learn more about 'static and check if it is valid here
    async fn modify<F, Fut>(&self, id: Uuid, modifier: F) -> Result<(), SubscriberError>
    where
        F: Fn(Subscriber) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Subscriber, SubscriberError>> + Send + 'static;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_status(
        &self,
        status: SubscriberStatus,
    ) -> Result<Vec<Subscriber>, SubscriberError>;
}
