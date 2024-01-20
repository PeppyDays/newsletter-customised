use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use uuid::Uuid;

use crate::api::error::ApiError;
use crate::api::runner::Container;
use crate::domain::subscription::subscriber::prelude::*;
use crate::domain::subscription::subscription_token::prelude::*;

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
    let subscriber = register_subscriber(id, request, container.subscriber_repository.clone())
        .await
        .map_err(|error| match error {
            SubscriberError::InvalidSubscriberName | SubscriberError::InvalidSubscriberEmail => {
                ApiError::new(StatusCode::BAD_REQUEST, error.into())
            }
            SubscriberError::RepositoryOperationFailed(_)
            | SubscriberError::MessengerOperationFailed(_)
            | SubscriberError::Unexpected(_) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into())
            }
        })?;

    let subscription_token =
        issue_subscription_token(&subscriber, container.subscription_token_repository.clone())
            .await
            .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    send_confirmation_email(
        &subscriber,
        &subscription_token,
        &container.exposing_address.url,
        container.subscriber_messenger.clone(),
    )
    .await
    .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(
    name = "Adding a new subscriber - register subscriber",
    skip(id, request, subscriber_repository)
)]
async fn register_subscriber(
    id: Uuid,
    request: Request,
    subscriber_repository: Arc<dyn SubscriberRepository>,
) -> Result<Subscriber, SubscriberError> {
    let subscriber = Subscriber::new(id, request.email, request.name)?;
    subscriber_repository.save(&subscriber).await?;
    Ok(subscriber)
}

#[tracing::instrument(
    name = "Adding a new subscriber - issue subscription token",
    skip(subscriber, subscription_token_repository)
)]
async fn issue_subscription_token(
    subscriber: &Subscriber,
    subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
) -> Result<SubscriptionToken, SubscriptionTokenError> {
    let subscription_token = SubscriptionToken::issue(subscriber.id);
    subscription_token_repository
        .save(&subscription_token)
        .await?;
    Ok(subscription_token)
}

#[tracing::instrument(
    name = "Adding a new subscriber - send confirmation email",
    skip(subscriber, subscription_token, exposing_url, subscriber_messenger)
)]
async fn send_confirmation_email(
    subscriber: &Subscriber,
    subscription_token: &SubscriptionToken,
    exposing_url: &str,
    subscriber_messenger: Arc<dyn SubscriberMessenger>,
) -> Result<(), SubscriberError> {
    let subscription_confirmation_url = format!(
        "{}/subscriptions/confirm?token={}",
        exposing_url, subscription_token.token,
    );

    subscriber_messenger
        .send(
            subscriber,
            "Welcome to our newsletter!",
            &format!(
                r#"Welcome to our newsletter! Click <a href="{}">here</a> to confirm your subscription."#,
                subscription_confirmation_url,
            ),
        ).await?;

    Ok(())
}
