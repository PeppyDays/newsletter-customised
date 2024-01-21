use std::sync::Arc;

use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::api::error::ApiError;
use crate::domain::subscription::subscriber::prelude::{
    Subscriber,
    SubscriberMessenger,
    SubscriberRepository,
    SubscriberStatus,
};

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Request {
    title: String,
    content: String,
}

#[tracing::instrument(
    name = "Publishing a newsletter",
    skip(subscriber_repository, subscriber_messenger)
)]
pub async fn handle(
    State(subscriber_repository): State<Arc<dyn SubscriberRepository>>,
    State(subscriber_messenger): State<Arc<dyn SubscriberMessenger>>,
    Json(request): Json<Request>,
) -> Result<StatusCode, ApiError> {
    let confirmed_subscribers: Vec<Subscriber> = subscriber_repository
        .find_by_status(SubscriberStatus::Confirmed)
        .await
        .context("Failed to get confirmed subscribers")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    if confirmed_subscribers.is_empty() {
        tracing::warn!("No confirmed subscribers found")
    }

    for subscriber in confirmed_subscribers {
        let response = subscriber_messenger
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
