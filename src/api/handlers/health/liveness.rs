use axum::{http::StatusCode, response::IntoResponse};

pub async fn handle() -> impl IntoResponse {
    StatusCode::OK
}
