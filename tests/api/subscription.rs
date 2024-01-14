use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::FirstName;
use fake::Fake;
use newsletter::domain::subscription::subscriber::model::SubscriberStatus;
use newsletter::domain::subscription::subscriber::repository::SubscriberRepository;
use reqwest::StatusCode;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::api::helper::app::App;

#[tokio::test]
async fn subscription_with_valid_form_returns_201() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // when
    let response = app.post_subscribe(&parameters).await;

    // then
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

#[tokio::test]
async fn subscription_sends_confirmation_email_for_validate_email_address() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // when
    app.post_subscribe(&parameters).await;

    // then
    // mock asserts on drop
}

#[tokio::test]
async fn subscription_sends_confirmation_email_with_link() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    // when
    app.post_subscribe(&parameters).await;

    // then
    let request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
    let content = body.get("Content").unwrap();

    assert!(content
        .as_str()
        .unwrap()
        .contains("/subscriptions/confirm?token="));
}

#[tokio::test]
async fn subscriber_is_confirmed_after_clicking_confirmation_link() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscribe(&parameters).await;

    // when
    let request = &app.email_server.received_requests().await.unwrap()[0];
    let body: serde_json::Value = serde_json::from_slice(&request.body).unwrap();
    let content = body.get("Content").unwrap();
    let token = content
        .as_str()
        .unwrap()
        .split("/subscriptions/confirm?token=")
        .collect::<Vec<&str>>()[1]
        .split('"')
        .collect::<Vec<&str>>()[0];

    let parameters = [("token", token)];
    app.get_subscription_confirm(&parameters).await;

    // then
    let saved_subscriber = app
        .subscriber_repository
        .find_by_email(email.as_str())
        .await
        .unwrap()
        .unwrap();

    assert!(matches!(
        saved_subscriber.status,
        SubscriberStatus::Confirmed,
    ));
}
