use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub struct ApiError {
    code: StatusCode,
    source: anyhow::Error,
}

impl ApiError {
    pub fn new(code: StatusCode, source: anyhow::Error) -> Self {
        Self { code, source }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorMessage {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.code,
            Json(ErrorMessage {
                error: self.source.to_string(),
            }),
        )
            .into_response()
    }
}
