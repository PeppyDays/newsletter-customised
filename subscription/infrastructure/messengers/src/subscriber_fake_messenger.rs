use domain::prelude::{
    Subscriber,
    SubscriberError,
    SubscriberMessenger,
};

#[derive(Clone)]
pub struct SubscriberFakeMessenger {}

impl SubscriberFakeMessenger {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait::async_trait]
impl SubscriberMessenger for SubscriberFakeMessenger {
    #[tracing::instrument(name = "Sending an fake message for subscription", skip(self))]
    async fn send(
        &self,
        recipient: &Subscriber,
        subject: &str,
        content: &str,
    ) -> Result<(), SubscriberError> {
        Ok(())
    }
}
