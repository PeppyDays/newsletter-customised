use fake::{
    faker::{internet::en::SafeEmail, name::en::FirstName},
    Fake,
};
use reqwest::StatusCode;

use newsletter::domain::subscriber::repository::SubscriberRepository;

use crate::api::helper::app::App;

#[tokio::test]
async fn subscription_with_valid_form_returns_201() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    // when
    let response = app.post_subscribe(&parameters).await;

    //then
    assert_eq!(response.status(), StatusCode::CREATED);

    let saved_subscriber = app
        .subscriber_repository
        .find_by_email(email.as_str())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_subscriber.email.as_ref(), email);
    assert_eq!(saved_subscriber.name.as_ref(), name);
}

#[tokio::test]
async fn subscription_with_missing_fields_returns_422() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters_set = [[("email", email.as_str())], [("name", name.as_str())]];

    for parameters in parameters_set {
        // when
        let response = app.post_subscribe(&parameters).await;

        // then
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn subscription_with_too_short_name_returns_400() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name = "s".to_string();
    let parameters = [("email", email), ("name", name)];

    // when
    let response = app.post_subscribe(&parameters).await;

    // then
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response.text().await.unwrap(),
        r#"{"error":"Subscriber's name is invalid"}"#,
    );
}
