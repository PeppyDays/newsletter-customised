use uuid::Uuid;

#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber's email verification status is invalid")]
    InvalidSubscriberEmailVerificationStatus,

    #[error("Subscriber's email is invalid")]
    InvalidSubscriberEmail,

    #[error("Subscriber (ID: {0}) doesn't exist")]
    SubscriberNotFound(Uuid),

    #[error("Failed to operator on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
