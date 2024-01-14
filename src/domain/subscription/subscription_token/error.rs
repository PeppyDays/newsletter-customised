#[derive(thiserror::Error, Debug)]
pub enum SubscriptionTokenError {
    #[error("Failed to issue a subscription token")]
    IssuanceFailed(#[source] anyhow::Error),

    #[error("Failed to operate on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}
