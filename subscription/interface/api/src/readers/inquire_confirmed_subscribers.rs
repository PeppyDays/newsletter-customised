use std::sync::Arc;

use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use domain::prelude::{
    Subscriber,
    SubscriberRepository,
    SubscriberStatus,
};

use crate::error::ApiError;

#[derive(serde::Serialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Response {
    name: String,
    email: String,
}

impl From<Subscriber> for Response {
    fn from(subscriber: Subscriber) -> Self {
        Response {
            name: subscriber.name.as_ref().to_owned(),
            email: subscriber.email.as_ref().to_owned(),
        }
    }
}

#[tracing::instrument(name = "Inquiring confirmed subscribers", skip(subscriber_repository))]
pub async fn read(
    State(subscriber_repository): State<Arc<dyn SubscriberRepository>>,
) -> Result<Json<Vec<Response>>, ApiError> {
    Ok(Json(
        subscriber_repository
            .find_by_status(SubscriberStatus::Confirmed)
            .await
            .context("Failed to get subscribers")
            .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?
            .into_iter()
            .map(Response::from)
            .collect(),
    ))
}
