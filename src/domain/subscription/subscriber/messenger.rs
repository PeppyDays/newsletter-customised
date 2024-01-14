use crate::domain::subscription::subscriber::error::SubscriberError;
use crate::domain::subscription::subscriber::model::Subscriber;

#[async_trait::async_trait]
pub trait SubscriberMessenger: Send + Sync {
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError>;
}
