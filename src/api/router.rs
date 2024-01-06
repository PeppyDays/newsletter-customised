use axum::{
    extract::MatchedPath,
    http::Request,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::api::handlers::{check_health, subscribe};
use crate::api::runner::Container;

pub async fn get_router(container: Container) -> Router {
    Router::new()
        .route("/subscribe", post(subscribe::handle))
        .with_state(container)
        .route("/", get(check_health::handle))
        .layer(
            // Refer to https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/Cargo.toml
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let correlation_id = Uuid::new_v4();
                let path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                tracing::info_span!(
                    "Processing HTTP request",
                    method = ?request.method(),
                    path,
                    correlation_id = %correlation_id,
                    causality_id = %correlation_id,
                )
            }),
        )
}
