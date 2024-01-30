use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use domain::prelude::{
    SubscriberMessenger,
    SubscriberRepository,
};
use uuid::Uuid;

use crate::runner::Container;

// TODO: Is there a better way to handle dependencies liveness check?
pub async fn handle<R, M>(State(container): State<Container<R, M>>) -> impl IntoResponse
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    // check subscriber repository
    let response = container
        .subscriber_repository
        .find_by_id(Uuid::new_v4())
        .await;

    if response.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    // check subscription token repository
    let response = container
        .subscription_token_repository
        .find_by_token("12345")
        .await;

    if response.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}
