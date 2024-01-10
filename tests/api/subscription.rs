use fake::{
    faker::{internet::en::SafeEmail, name::en::FirstName},
    Fake,
};
use reqwest::StatusCode;

use newsletter::domain::subscriber::SubscriberRepository;

use crate::api::helper::app::App;

#[tokio::test]
async fn subscription_with_valid_form_returns_200() {
    // given
    let app = App::new().await;

    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    // when
    let response = app.post_subscribe(&parameters).await;

    //then
    assert_eq!(response.status(), StatusCode::OK);

    let saved_subscriber = app
        .subscriber_repository
        .find_by_email(email.as_str())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(saved_subscriber.email.as_str(), email.as_str());
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
