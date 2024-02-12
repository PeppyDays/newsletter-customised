use axum::extract::MatchedPath;
use axum::http::Request;
use axum::routing::get;
use axum::Router;
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use domain::prelude::SubscriberRepository;

use crate::container::Container;
use crate::{
    checkers,
    document,
};

pub async fn get_router<R>(container: Container<R>) -> Router
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url(
            "/api-docs/openapi.json",
            document::OpenApiDocument::openapi(),
        ))
        .route("/checkers/readiness", get(checkers::readiness::handle))
        .with_state(container)
        .route("/checkers/liveness", get(checkers::liveness::handle))
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
