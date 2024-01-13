use crate::domain::subscriber::{
    error::SubscriberError, messenger::SubscriberMessenger, model::Subscriber,
};

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
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError> {
        let url = self
            .host
            .join("email")
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
            .map_err(|error| SubscriberError::MessengerOperationFailed(error.into()))?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
struct Request<'a> {
    sender: &'a str,
    recipient: &'a str,
    subject: &'a str,
    content: &'a str,
}

#[cfg(test)]
mod tests {
    use fake::{
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
            name::en::FirstName,
        },
        Fake,
    };
    use uuid::Uuid;
    use wiremock::{
        matchers::{header, header_exists, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        domain::subscriber::{messenger::SubscriberMessenger, model::Subscriber},
        infrastructure::messengers::SubscriberEmailMessenger,
    };

    #[tokio::test]
    async fn send_email_fires_request_to_email_server() {
        // given
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

        Mock::given(path("/email"))
            .and(header_exists(reqwest::header::AUTHORIZATION))
            .and(header(reqwest::header::CONTENT_TYPE, "application/json"))
            .and(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&email_server)
            .await;

        // when
        let subscriber =
            Subscriber::new(Uuid::new_v4(), SafeEmail().fake(), FirstName().fake()).unwrap();
        let subject: String = Sentence(1..2).fake();
        let content: String = Paragraph(1..10).fake();

        messenger
            .send(&subscriber, &subject, &content)
            .await
            .unwrap();

        // then
        // no error is expected
    }
}
