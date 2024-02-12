use axum::http::StatusCode;
use axum::response::IntoResponse;

#[utoipa::path(get, path = "/checkers/liveness", responses(
    (status = 200, description = "Check livenss of API service")
))]
pub async fn handle() -> impl IntoResponse {
    StatusCode::OK
}
