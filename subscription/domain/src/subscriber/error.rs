use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber's name is invalid")]
    InvalidSubscriberName,

    #[error("Subscriber's email is invalid")]
    InvalidSubscriberEmail,

    #[error("Subscriber's status is invalid")]
    InvalidSubscriberStatus,

    #[error("Subscriber (ID: {0}) doesn't exist")]
    SubscriberNotFound(Uuid),

    #[error("Failed to operate on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed to send a message through messenger")]
    MessengerOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
