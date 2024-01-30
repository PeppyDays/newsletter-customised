use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::{
    get,
    post,
};
use axum::Router;
use domain::prelude::{
    SubscriberMessenger,
    SubscriberRepository,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::runner::Container;
use crate::{
    checkers,
    executors,
    readers,
};

pub async fn get_router<R, M>(container: Container<R, M>) -> Router
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    Router::new()
        .route(
            "/subscription/query/inquire-confirmed-subscribers/read",
            get(readers::inquire_confirmed_subscribers::read),
        )
        .route(
            "/subscription/command/confirm/execute",
            post(executors::confirm::execute),
        )
        .route(
            "/subscription/command/subscribe/execute",
            post(executors::subscribe::execute),
        )
        .route("/health/readiness", get(checkers::readiness::handle))
        .with_state(container)
        .route("/health/liveness", get(checkers::liveness::handle))
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
