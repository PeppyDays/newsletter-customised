use axum::{extract::State, http::StatusCode, response::IntoResponse, Form};
use uuid::Uuid;

use crate::{api::runner::Container, domain::subscriber::Subscriber};

#[derive(serde::Deserialize, Debug)]
pub struct Request {
    email: String,
    name: String,
}

#[tracing::instrument(name = "Adding a new subscriber", skip(container))]
pub async fn handle(
    State(container): State<Container>,
    Form(request): Form<Request>,
) -> impl IntoResponse {
    let id = Uuid::new_v4();
    // TODO: Use a proper error type
    let subscriber = Subscriber::new(id, request.email, request.name).unwrap();

    match container.subscriber_repository.save(&subscriber).await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            // TODO: Modify simple println to tracing event
            println!("Failed to save a subscriber: {:?}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
