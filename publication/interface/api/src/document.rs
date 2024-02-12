use utoipa::OpenApi;

use crate::checkers;

// TODO: utoipa crate doesn't support State with impl trait
// TODO: Search auto search for handlers
#[derive(OpenApi)]
#[openapi(
    paths(
        checkers::liveness::handle,
        checkers::readiness::handle,
    ),
    tags(
        (name = "Publication", description = "Publishing APIs of newsletter to subscribers")
    )
)]
pub struct OpenApiDocument;
