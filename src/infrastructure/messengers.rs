use crate::domain::subscriber::{
    error::SubscriberError, messenger::SubscriberMessenger, model::Subscriber,
};

#[derive(Clone)]
pub struct SubscriberEmailMessenger {
    client: reqwest::Client,
    host: String,
    sender: String,
}

impl SubscriberEmailMessenger {
    pub fn new(client: reqwest::Client, host: String, sender: String) -> Self {
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
        todo!();
    }
}
