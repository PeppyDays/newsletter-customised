use crate::domain::subscription::subscriber::error::SubscriberError;
use crate::domain::subscription::subscriber::messenger::SubscriberMessenger;
use crate::domain::subscription::subscriber::model::Subscriber;

#[derive(Clone)]
pub struct SubscriberEmailMessenger {
    client: reqwest::Client,
    host: reqwest::Url,
    sender: String,
}

impl SubscriberEmailMessenger {
    pub fn new(client: reqwest::Client, host: reqwest::Url, sender: String) -> Self {
        Self {
            client,
            host,
            sender,
        }
    }
}

#[async_trait::async_trait]
impl SubscriberMessenger for SubscriberEmailMessenger {
    #[tracing::instrument(name = "Sending an email for subscription", skip(self))]
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError> {
        let url = self
            .host
            .join("/email")
            .map_err(|error| SubscriberError::MessengerOperationFailed(error.into()))?;
        let body = Request {
            sender: self.sender.as_ref(),
            recipient: recipient.email.as_ref(),
            subject,
            content,
        };

        self.client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|error| SubscriberError::MessengerOperationFailed(error.into()))?
            .error_for_status()
            .map_err(|error| SubscriberError::MessengerOperationFailed(error.into()))?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct Request<'a> {
    sender: &'a str,
    recipient: &'a str,
    subject: &'a str,
    content: &'a str,
}

#[cfg(test)]
mod tests {
    use claims::{
        assert_err,
        assert_ok,
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{
        Paragraph,
        Sentence,
    };
    use fake::faker::name::en::FirstName;
    use fake::Fake;
    use uuid::Uuid;
    use wiremock::matchers::{
        any,
        header,
        header_exists,
        method,
        path,
    };
    use wiremock::{
        Mock,
        MockServer,
        ResponseTemplate,
    };

    use crate::domain::subscription::subscriber::messenger::SubscriberMessenger;
    use crate::domain::subscription::subscriber::model::Subscriber;
    use crate::infrastructure::messengers::SubscriberEmailMessenger;

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);

            if let Ok(body) = result {
                body.get("Sender").is_some()
                    && body.get("Recipient").is_some()
                    && body.get("Subject").is_some()
                    && body.get("Content").is_some()
            } else {
                false
            }
        }
    }

    async fn run_email_server() -> (MockServer, SubscriberEmailMessenger) {
        let email_server = MockServer::start().await;
        let sender: String = SafeEmail().fake();

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_static("welcome"),
        );
        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .expect("Failed to create email client pool");
        let messenger = SubscriberEmailMessenger::new(
            client,
            reqwest::Url::parse(email_server.uri().as_ref()).unwrap(),
            sender,
        );

        (email_server, messenger)
    }

    #[tokio::test]
    async fn send_email_fires_request_to_email_server() {
        // given
        let (email_server, messenger) = run_email_server().await;

        let subscriber =
            Subscriber::new(Uuid::new_v4(), SafeEmail().fake(), FirstName().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(path("/email"))
            .and(header_exists(reqwest::header::AUTHORIZATION))
            .and(header(reqwest::header::CONTENT_TYPE, "application/json"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&email_server)
            .await;

        // when
        messenger
            .send(&subscriber, &subject, &content)
            .await
            .unwrap();

        // then
        // no error is expected
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_returns_200() {
        // given
        let (email_server, messenger) = run_email_server().await;

        let subscriber =
            Subscriber::new(Uuid::new_v4(), SafeEmail().fake(), FirstName().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&email_server)
            .await;

        // when
        let response = messenger.send(&subscriber, &subject, &content).await;

        // then
        assert_ok!(response);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_returns_500() {
        // given
        let (email_server, messenger) = run_email_server().await;

        let subscriber =
            Subscriber::new(Uuid::new_v4(), SafeEmail().fake(), FirstName().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&email_server)
            .await;

        // when
        let response = messenger.send(&subscriber, &subject, &content).await;

        // then
        assert_err!(response);
    }
}
