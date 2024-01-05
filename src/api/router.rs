use axum::{
    routing::{get, post},
    Router,
};

use crate::api::handlers::{check_health, subscribe};
use crate::api::runner::Container;

pub async fn get_router(container: Container) -> Router {
    Router::new()
        .route("/subscribe", post(subscribe::handle))
        .with_state(container)
        .route("/", get(check_health::handle))
}
