use uuid::Uuid;

pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl Subscriber {
    pub fn new(id: Uuid, email: String, name: String) -> Self {
        Self { id, email, name }
    }
}

#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError>;
}

#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Failed to operate on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
