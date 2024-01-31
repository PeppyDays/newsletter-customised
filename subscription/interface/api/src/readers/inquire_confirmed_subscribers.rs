use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use domain::prelude::{
    Subscriber,
    SubscriberQuery,
    SubscriberQueryReader,
    SubscriberRepository,
};

use crate::error::ApiError;

#[readonly::make]
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

#[tracing::instrument(
    name = "Inquiring confirmed subscribers",
    skip(subscriber_query_reader)
)]
pub async fn read(
    State(subscriber_query_reader): State<SubscriberQueryReader<impl SubscriberRepository>>,
) -> Result<Json<Vec<Response>>, ApiError> {
    let inquire_confirmed_subscribers_query = SubscriberQuery::InquireConfirmedSubscribers;
    Ok(Json(
        subscriber_query_reader
            .read(inquire_confirmed_subscribers_query)
            .await
            .context("Failed to get subscribers")
            .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?
            .into_iter()
            .map(Response::from)
            .collect(),
    ))
}
