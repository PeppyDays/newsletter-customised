use crate::domain::subscriber::{
    error::SubscriberError, messenger::SubscriberMessenger, model::Subscriber,
};

#[derive(Clone)]
pub struct SubscriberEmailMessenger {
    client: reqwest::Client,
}

impl SubscriberEmailMessenger {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
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
        todo!();
    }
}
