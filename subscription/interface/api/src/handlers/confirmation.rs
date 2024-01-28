use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use domain::prelude::{SubscriberRepository, SubscriptionTokenRepository};

use crate::error::ApiError;

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    token: String,
}

#[tracing::instrument(
    name = "Confirming a subscription",
    skip(subscriber_repository, subscription_token_repository)
)]
pub async fn handle(
    State(subscriber_repository): State<Arc<dyn SubscriberRepository>>,
    State(subscription_token_repository): State<Arc<dyn SubscriptionTokenRepository>>,
    Query(request): Query<Request>,
) -> Result<StatusCode, ApiError> {
    let subscription_token = subscription_token_repository
        .find_by_token(&request.token)
        .await
        .context("Failed to get subscription token")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("No subscription token found for the given subscription token"),
            )
        })?;
    let subscriber_id = subscription_token.subscriber_id;

    let mut subscriber = subscriber_repository
        .find_by_id(subscriber_id)
        .await
        .context("Failed to get subscriber")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?
        .ok_or_else(|| {
            ApiError::new(
                StatusCode::NOT_FOUND,
                anyhow::anyhow!("No subscriber found from the given subscription token"),
            )
        })?;
    subscriber.confirm();

    subscriber_repository
        .save(&subscriber)
        .await
        .context("Failed to make the subscriber confirmed")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    Ok(StatusCode::OK)
}

#[cfg(test)]
mod tests {
    use domain::prelude::{
        MockSubscriberRepository, MockSubscriptionTokenRepository, Subscriber, SubscriptionToken,
    };
    use fake::{
        faker::{internet::en::SafeEmail, name::en::FirstName},
        Fake,
    };
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn confirmation_with_not_existing_token_returns_not_found() {
        // given
        let subscriber_repository = MockSubscriberRepository::new();
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();

        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(|_| Ok(Option::None));

        // when
        let request = Request {
            token: "not-existing-token".to_string(),
        };
        let response = handle(
            State(Arc::new(subscriber_repository)),
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
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();

        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(|_| Ok(Option::Some(SubscriptionToken::issue(Uuid::new_v4()))));

        subscriber_repository
            .expect_find_by_id()
            .once()
            .returning(|_| Ok(Option::None));

        // when
        let request = Request {
            token: "existing-token".to_string(),
        };
        let response = handle(
            State(Arc::new(subscriber_repository)),
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
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();

        let subscriber_id = Uuid::new_v4();

        subscription_token_repository
            .expect_find_by_token()
            .once()
            .returning(move |_| Ok(Option::Some(SubscriptionToken::issue(subscriber_id))));

        subscriber_repository
            .expect_find_by_id()
            .once()
            .returning(move |_| {
                Ok(Option::Some(
                    Subscriber::new(subscriber_id, SafeEmail().fake(), FirstName().fake()).unwrap(),
                ))
            });

        subscriber_repository
            .expect_save()
            .once()
            .returning(|_| Ok(()));

        // when
        let request = Request {
            token: "existing-token".to_string(),
        };
        let response = handle(
            State(Arc::new(subscriber_repository)),
            State(Arc::new(subscription_token_repository)),
            Query(request),
        )
        .await;

        // then
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::OK)
    }
}
