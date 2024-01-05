use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use uuid::Uuid;

use crate::{api::runner::Container, domain::subscriber::Subscriber};

#[derive(serde::Deserialize)]
pub struct Request {
    email: String,
    name: String,
}

pub async fn handle(
    State(container): State<Container>,
    Form(request): Form<Request>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    let subscriber = Subscriber {
        id,
        email: request.email,
        name: request.name,
    };

    match container.subscriber_repository.save(&subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            println!("Failed to save a subscriber: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
