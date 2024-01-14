use axum::http::StatusCode;
use axum::response::IntoResponse;

// TODO: modify the inside to use this as readiness check
pub async fn handle() -> impl IntoResponse {
    StatusCode::OK
}
