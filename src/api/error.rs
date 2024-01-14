use std::fmt::{Display, Formatter};

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

    pub fn source(&self) -> &anyhow::Error {
        &self.source
    }
}

impl Display for ApiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self.source(), f)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct ErrorMessage {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        tracing::error!("{:?}", self);

        (
            self.code,
            Json(ErrorMessage {
                error: self.source.to_string(),
            }),
        )
            .into_response()
    }
}

fn error_chain_fmt(
    // e: &impl std::error::Error,
    e: &anyhow::Error,
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
