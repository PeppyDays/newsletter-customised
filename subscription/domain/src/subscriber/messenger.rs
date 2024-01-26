use crate::subscriber::error::SubscriberError;
use crate::subscriber::model::Subscriber;

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriberMessenger: Send + Sync {
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError>;
}
