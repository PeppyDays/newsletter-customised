use axum::http::StatusCode;
use axum::response::IntoResponse;

pub async fn handle() -> impl IntoResponse {
    StatusCode::OK
}
