use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
use crate::subscriber::messenger::SubscriberMessenger;
use crate::subscriber::model::{
    Subscriber,
    SubscriberEmail,
    SubscriberName,
};
use crate::subscriber::repository::SubscriberRepository;

pub enum SubscriberCommand {
    RegisterSubscriber {
        id: Uuid,
        email: String,
        name: String,
    },
    SendConfirmationMessage {
        id: Uuid,
        token: String,
    },
}

#[derive(Clone)]
pub struct SubscriberCommandExecutor<R: SubscriberRepository, M: SubscriberMessenger> {
    repository: R,
    messenger: M,
    exposing_address: String,
}

impl<R, M> SubscriberCommandExecutor<R, M>
where
    R: SubscriberRepository,
    M: SubscriberMessenger,
{
    pub fn new(repository: R, messenger: M, exposing_address: String) -> Self {
        Self {
            repository,
            messenger,
            exposing_address,
        }
    }

    pub async fn execute(&self, command: SubscriberCommand) -> Result<(), SubscriberError> {
        match command {
            SubscriberCommand::RegisterSubscriber { id, email, name } => {
                let name = SubscriberName::parse(name)?;
                let email = SubscriberEmail::parse(email)?;
                let subscriber = Subscriber::new(id, email, name);

                self.repository.save(&subscriber).await
            }
            SubscriberCommand::SendConfirmationMessage { id, token } => {
                let subscriber = self
                    .repository
                    .find_by_id(id)
                    .await?
                    .ok_or(SubscriberError::SubscriberNotFound(id))?;

                let confirmation_url = format!(
                    "{}/subscriptions/confirm?token={}",
                    self.exposing_address, token,
                );
                let title = "Welcome to our newsletter!";
                let content = &format!(
                    r#"Welcome to our newsletter! Click <a href="{}">here</a> to confirm your subscription."#,
                    confirmation_url
                );

                self.messenger.send(&subscriber, title, content).await
            }
        }
    }
}
