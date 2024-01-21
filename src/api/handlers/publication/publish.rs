use anyhow::Context;
use axum::{extract::State, http::StatusCode, Json};

use crate::{
    api::{error::ApiError, runner::Container},
    domain::subscription::subscriber::prelude::{Subscriber, SubscriberStatus},
};

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    title: String,
    content: String,
}

#[tracing::instrument(name = "Publishing a newsletter", skip(container))]
pub async fn handle(
    State(container): State<Container>,
    Json(request): Json<Request>,
) -> Result<StatusCode, ApiError> {
    let confirmed_subscribers: Vec<Subscriber> = container
        .subscriber_repository
        .find_by_status(SubscriberStatus::Confirmed)
        .await
        .context("Failed to get confirmed subscribers")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    if confirmed_subscribers.is_empty() {
        tracing::warn!("No confirmed subscribers found")
    }

    for subscriber in confirmed_subscribers {
        let response = container
            .subscriber_messenger
            .send(&subscriber, &request.title, &request.content)
            .await;

        if let Err(error) = response {
            tracing::error!(
                "Failed to send newsletter to {:?} due to the error {:?}",
                subscriber,
                error
            );
        }
    }

    Ok(StatusCode::OK)
}
