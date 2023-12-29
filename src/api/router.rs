use axum::{routing::get, Router};

use crate::api::handlers::check_health;

pub async fn get_router() -> Router {
    Router::new().route("/", get(check_health::handle))
}
