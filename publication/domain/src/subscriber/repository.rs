use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::Subscriber;

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    // async fn modify<F, Fut>(&self, id: Uuid, modifier: F) -> Result<(), SubscriberError>
    // where
    //     F: Fn(Subscriber) -> Fut + Send + 'static,
    //     Fut: Future<Output = Result<Subscriber, SubscriberError>> + Send + 'static;
    async fn modify(
        &self,
        id: Uuid,
        modifier: fn(Subscriber) -> Result<Subscriber, SubscriberError>,
    ) -> Result<(), SubscriberError>;
    async fn find_all(&self) -> Result<Vec<Subscriber>, SubscriberError>;
}
