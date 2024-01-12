use crate::domain::subscriber::error::SubscriberError;
use crate::domain::subscriber::model::Subscriber;

#[async_trait::async_trait]
pub trait SubscriberMessenger: Send + Sync {
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError>;
}
