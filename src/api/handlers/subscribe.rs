use axum::{extract::State, http::StatusCode, Form};
use uuid::Uuid;

use crate::{
    api::{error::ApiError, runner::Container},
    domain::subscriber::{error::SubscriberError, model::Subscriber},
};

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
            SubscriberError::RepositoryOperationFailed(_) | SubscriberError::Unexpected(_) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into())
            }
        })?;

    container
        .subscriber_repository
        .save(&subscriber)
        .await
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into()))?;

    Ok(StatusCode::CREATED)
}
