use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::FirstName;
use fake::Fake;
use newsletter::domain::subscription::subscriber::prelude::{
    Subscriber,
    SubscriberRepository,
};
use newsletter::domain::subscription::subscription_token::prelude::SubscriptionTokenRepository;
use wiremock::matchers::{
    method,
    path,
};
use wiremock::{
    Mock,
    ResponseTemplate,
};

use crate::api::helper::app::App;

pub async fn create_unconfirmed_subscriber(app: &App) -> Subscriber {
    let email: String = SafeEmail().fake();
    let name: String = FirstName().fake();
    let parameters = [("email", email.as_str()), ("name", name.as_str())];

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscription_subscribe(&parameters)
        .await
        .error_for_status()
        .unwrap();

    let _request_in_email_server = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.subscriber_repository
        .find_by_email(&email)
        .await
        .unwrap()
        .unwrap()
}

pub async fn create_confirmed_subscriber(app: &App) -> Subscriber {
    let subscriber = create_unconfirmed_subscriber(app).await;
    let subscription_token = app
        .subscription_token_repository
        .find_by_subscriber_id(subscriber.id)
        .await
        .unwrap()
        .unwrap();
    let parameters = [("token", subscription_token.token)];

    app.get_subscription_confirm(&parameters).await;

    app.subscriber_repository
        .find_by_id(subscriber.id)
        .await
        .unwrap()
        .unwrap()
}
