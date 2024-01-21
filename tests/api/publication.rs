use reqwest::StatusCode;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helper::app::App;
use crate::api::helper::cases::{
    create_confirmed_subscriber,
    create_unconfirmed_subscriber,
};

#[tokio::test]
async fn newsletters_are_not_delivered_when_title_or_content_is_empty() {
    // given
    let app = App::new().await;
    let wrong_requests = vec![
        serde_json::json!({"Title": "Hi!"}),
        serde_json::json!({"Content": "Hi!"}),
    ];

    for request in wrong_requests {
        // when
        let response = app.post_publication_publish(&request).await;

        // then
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // given
    let app = App::new().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // when
    let request = serde_json::json!({
        "Title": "Hello",
        "Content": "Good to see you :)",
    });
    let response = app.post_publication_publish(&request).await;

    // then
    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // given
    let app = App::new().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // when
    let reqest_body = serde_json::json!({
        "Title": "Hello",
        "Content": "Good to see you :)",
    });
    let response = app.post_publication_publish(&reqest_body).await;

    // then
    assert_eq!(response.status(), StatusCode::OK)
}
