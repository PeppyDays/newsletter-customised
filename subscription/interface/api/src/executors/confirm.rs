use std::sync::Arc;

use anyhow::Context;
use axum::extract::{
    Query,
    State,
};
use axum::http::StatusCode;
use domain::prelude::{
    SubscriberCommand,
    SubscriberCommandExecutor,
    SubscriberError,
    SubscriberMessenger,
    SubscriberRepository,
    SubscriptionTokenRepository,
};

use crate::error::ApiError;

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    token: String,
}

#[tracing::instrument(
    name = "Confirming a subscription",
    skip(subscriber_command_executor, subscription_token_repository)
)]
pub async fn execute(
    State(subscriber_command_executor): State<
        SubscriberCommandExecutor<impl SubscriberRepository, impl SubscriberMessenger>,
    >,
    State(subscription_token_repository): State<Arc<dyn SubscriptionTokenRepository>>,
    Query(request): Query<Request>,
) -> Result<StatusCode, ApiError> {
    let subscription_token = subscription_token_repository
        .find_by_token(&request.token)
        .await
        .context("Failed to get subscription token")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or(ApiError::new(
            StatusCode::NOT_FOUND,
            anyhow::anyhow!("No subscription token found for the given subscription token"),
        ))?;

    let subscriber_id = subscription_token.subscriber_id;
    let confirm_subscription_command = SubscriberCommand::ConfirmSubscription { id: subscriber_id };
    subscriber_command_executor
        .execute(confirm_subscription_command)
        .await
        .map_err(|error| match error {
            SubscriberError::SubscriberNotFound(_) => ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("No subscriber found for the given subscription token"),
            ),
            _ => ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()),
        })?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use domain::prelude::{
        MockSubscriberMessenger,
        MockSubscriberRepository,
        MockSubscriptionTokenRepository,
        Subscriber,
        SubscriptionToken,
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::FirstName;
    use fake::Fake;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn confirmation_with_not_existing_token_returns_not_found() {
        // given
        let subscriber_repository = MockSubscriberRepository::new();
        let subscriber_messenger = MockSubscriberMessenger::new();
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();
        let exposing_address = "http://localhost:3000".to_string();

        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(|_| Ok(Option::None));

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );

        // when
        let request = Request {
            token: "not-existing-token".to_string(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(Arc::new(subscription_token_repository)),
            Query(request),
        )
        .await;

        // then
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn confirmation_with_existing_token_without_subscriber_returns_not_found() {
        // given
        let mut subscriber_repository = MockSubscriberRepository::new();
        let subscriber_messenger = MockSubscriberMessenger::new();
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();
        let exposing_address = "http://localhost:3000".to_string();

        subscriber_repository
            .expect_find_by_id()
            .once()
            .returning(|_| Ok(Option::None));
        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(|_| Ok(Option::Some(SubscriptionToken::issue(Uuid::new_v4()))));

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );

        // when
        let request = Request {
            token: "existing-token".to_string(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(Arc::new(subscription_token_repository)),
            Query(request),
        )
        .await;

        // then
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code, StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn confirmation_with_existing_token_and_subscriber_returns_ok() {
        // given
        let mut subscriber_repository = MockSubscriberRepository::new();
        let subscriber_messenger = MockSubscriberMessenger::new();
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();
        let exposing_address = "http://localhost:3000".to_string();
        let subscriber_id = Uuid::new_v4();

        subscriber_repository
            .expect_find_by_id()
            .once()
            .returning(move |_| {
                Ok(Option::Some(
                    Subscriber::register(subscriber_id, SafeEmail().fake(), FirstName().fake())
                        .unwrap(),
                ))
            });
        subscriber_repository
            .expect_save()
            .once()
            .returning(|_| Ok(()));
        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(move |_| Ok(Option::Some(SubscriptionToken::issue(subscriber_id))));

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );

        // when
        let request = Request {
            token: "existing-token".to_string(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(Arc::new(subscription_token_repository)),
            Query(request),
        )
        .await;

        // then
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::OK)
    }
}
