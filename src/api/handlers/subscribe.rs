use axum::{http::StatusCode, response::IntoResponse, Form};

#[derive(serde::Deserialize)]
pub struct Request {
    email: String,
    name: String,
}

pub async fn handle(Form(request): Form<Request>) -> impl IntoResponse {
    println!("email: {}", request.email);
    println!("name: {}", request.name);

    StatusCode::OK
}
