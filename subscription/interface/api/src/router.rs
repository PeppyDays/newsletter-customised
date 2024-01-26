use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{
    get,
    post,
};
use axum::Router;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::handlers::{
    confirmation,
    liveness,
    readiness,
    subscription,
};
use crate::runner::Container;

pub async fn get_router(container: Container) -> Router {
    Router::new()
        .route("/subscription/confirm", get(confirmation::handle))
        .route("/subscription/subscribe", post(subscription::handle))
        .route("/health/readiness", get(readiness::handle))
        .with_state(container)
        .route("/health/liveness", get(liveness::handle))
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
