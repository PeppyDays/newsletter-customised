use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::runner::Container;
use crate::domain::subscription::subscriber::prelude::{Subscriber, SubscriberError};
use crate::domain::subscription::subscription_token::prelude::SubscriptionToken;

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    email: String,
    name: String,
}

#[tracing::instrument(name = "Adding a new subscriber", skip(container))]
pub async fn handle(
    State(container): State<Container>,
    Form(request): Form<Request>,
) -> Result<StatusCode, ApiError> {
    let id = Uuid::new_v4();

    let subscriber =
        Subscriber::new(id, request.email, request.name).map_err(|error| match error {
            SubscriberError::InvalidSubscriberName => {
                ApiError::new(StatusCode::BAD_REQUEST, error.into())
            }
            SubscriberError::InvalidSubscriberEmail => {
                ApiError::new(StatusCode::BAD_REQUEST, error.into())
            }
            SubscriberError::RepositoryOperationFailed(_)
            | SubscriberError::MessengerOperationFailed(_)
            | SubscriberError::Unexpected(_) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into())
            }
        })?;

    container
        .subscriber_repository
        .save(&subscriber)
        .await
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    let subscription_token = SubscriptionToken::issue(subscriber.id);

    container
        .subscription_token_repository
        .save(&subscription_token)
        .await
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    let subscription_confirmation_url = format!(
        "{}/subscriptions/confirm?token={}",
        container.exposing_address.url, subscription_token.token,
    );

    container
        .subscriber_messenger
        .send(
            &subscriber,
            "Welcome to our newsletter!",
            &format!(
                r#"Welcome to our newsletter! Click <a href="{}">here</a> to confirm your subscription."#,
                subscription_confirmation_url,
            ),
        )
        .await
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    Ok(StatusCode::CREATED)
}
