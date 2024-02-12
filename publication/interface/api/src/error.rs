use std::fmt::{
    Debug,
    Formatter,
};

use axum::http::StatusCode;
use axum::response::{
    IntoResponse,
    Response,
};
use axum::Json;

#[derive(thiserror::Error)]
pub enum ApiError {
    #[error("Internal Server Error")]
    Unexpected(#[source] anyhow::Error),
}

impl Debug for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorMessage {
    code: String,
    message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);

        let response = match self {
            ApiError::Unexpected(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorMessage {
                    code: "Unexpected".to_string(),
                    message: error.to_string(),
                }),
            ),
        };
        response.into_response()
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
