use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;

use crate::api::runner::Container;

pub async fn handle(State(container): State<Container>) -> impl IntoResponse {
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
