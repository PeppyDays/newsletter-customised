use axum::{
    routing::{get, post},
    Router,
};

use crate::api::handlers::{check_health, subscribe};

pub async fn get_router() -> Router {
    Router::new()
        .route("/", get(check_health::handle))
        .route("/subscribe", post(subscribe::handle))
}
