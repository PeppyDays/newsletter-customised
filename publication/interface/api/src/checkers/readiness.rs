use axum::http::StatusCode;
use axum::response::IntoResponse;

// TODO: Is there a better way to handle dependencies liveness check?
#[utoipa::path(get, path = "/checkers/readiness", responses(
    (status = 200, description = "Check readiness of API service")
))]
pub async fn handle() -> impl IntoResponse {
    // // check subscriber repository
    // let response = container
    //     .subscriber_repository
    //     .find_by_id(Uuid::new_v4())
    //     .await;

    // if response.is_err() {
    //     return StatusCode::INTERNAL_SERVER_ERROR;
    // }

    // // check subscription token repository
    // let response = container
    //     .subscription_token_repository
    //     .find_by_token("12345")
    //     .await;

    // if response.is_err() {
    //     return StatusCode::INTERNAL_SERVER_ERROR;
    // }

    StatusCode::OK
}
