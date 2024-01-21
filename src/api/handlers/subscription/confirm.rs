use anyhow::Context;
use axum::extract::{
    Query,
    State,
};
use axum::http::StatusCode;

use crate::api::error::ApiError;
use crate::api::runner::Container;

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    token: String,
}

#[tracing::instrument(name = "Confirming a subscription", skip(container))]
pub async fn handle(
    State(container): State<Container>,
    Query(request): Query<Request>,
) -> Result<(), ApiError> {
    let subscription_token = container
        .subscription_token_repository
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

    let mut subscriber = container
        .subscriber_repository
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

    container
        .subscriber_repository
        .save(&subscriber)
        .await
        .context("Failed to make the subscriber confirmed")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    Ok(())
}
