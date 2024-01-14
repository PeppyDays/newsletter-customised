#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber's name is invalid")]
    InvalidSubscriberName,

    #[error("Subscriber's email is invalid")]
    InvalidSubscriberEmail,

    #[error("Failed to operate on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed to send a message through messenger")]
    MessengerOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
