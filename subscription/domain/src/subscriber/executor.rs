use uuid::Uuid;

use crate::subscriber::error::SubscriberError;
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
}

#[derive(Clone)]
pub struct SubscriberCommandExecutor<R: SubscriberRepository> {
    repository: R,
}

impl<R: SubscriberRepository> SubscriberCommandExecutor<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: SubscriberCommand) -> Result<(), SubscriberError> {
        match command {
            SubscriberCommand::RegisterSubscriber { id, email, name } => {
                let name = SubscriberName::parse(name)?;
                let email = SubscriberEmail::parse(email)?;
                let subscriber = Subscriber::new(id, email, name);

                self.repository.save(&subscriber).await
            }
        }
    }
}
