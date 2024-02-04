#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber's email verification status is invalid")]
    InvalidSubscriberEmailVerificationStatus,

    #[error("Failed to operator on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
