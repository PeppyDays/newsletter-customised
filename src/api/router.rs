use axum::{
    extract::MatchedPath,
    http::Request,
    routing::{get, post},
    Router,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::api::handlers::{health, subscription};
use crate::api::runner::Container;

pub async fn get_router(container: Container) -> Router {
    Router::new()
        .route("/subscription/confirm", get(subscription::confirm::handle))
        .route(
            "/subscription/subscribe",
            post(subscription::subscribe::handle),
        )
        .with_state(container)
        .route("/", get(health::check::handle))
        .layer(
            // Refer to https://github.com/tokio-rs/axum/blob/main/examples/tracing-aka-logging/Cargo.toml
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let path = request
                    .extensions()
                    .get::<MatchedPath>()
                    .map(MatchedPath::as_str);

                tracing::info_span!(
                    "Processing HTTP request",
                    method = ?request.method(),
                    path,
                    correlation_id = %Uuid::new_v4(),
                )
            }),
        )
}
