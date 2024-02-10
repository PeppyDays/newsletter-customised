use uuid::Uuid;

use crate::subscriber::model::prelude::{
    SubscriberError,
    SubscriberRepository,
};
use crate::subscriber::prelude::{
    Subscriber,
    SubscriberEmailVerifiationStatus,
};

pub enum SubscriberCommand {
    CreateSubscriber(CreateSubscriber),
    UpdateSubscriber(UpdateSubscriber),
    VerifySubscriberEmailAs(VerifySubscriberEmailAs),
}

pub struct SubscriberCommandExecutor<R>
where
    R: SubscriberRepository,
{
    repository: R,
}

impl<R> SubscriberCommandExecutor<R>
where
    R: SubscriberRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: SubscriberCommand) -> Result<(), SubscriberError> {
        match command {
            SubscriberCommand::CreateSubscriber(command) => {
                command.execute(self.repository.clone()).await
            }
            SubscriberCommand::UpdateSubscriber(command) => {
                command.execute(self.repository.clone()).await
            }
            SubscriberCommand::VerifySubscriberEmailAs(command) => {
                command.execute(self.repository.clone()).await
            }
        }
    }
}

pub struct CreateSubscriber {
    id: Uuid,
    email: String,
    name: String,
}

impl CreateSubscriber {
    pub fn new(id: Uuid, email: String, name: String) -> Self {
        Self { id, email, name }
    }

    async fn execute(&self, repository: impl SubscriberRepository) -> Result<(), SubscriberError> {
        let mut subscriber = Subscriber::default();
        subscriber.create(self.id, self.email.clone(), self.name.clone());

        repository.save(&mut subscriber).await
    }
}

pub struct UpdateSubscriber {
    id: Uuid,
    name: String,
}

impl UpdateSubscriber {
    pub fn new(id: Uuid, name: String) -> Self {
        Self { id, name }
    }

    async fn execute(&self, repository: impl SubscriberRepository) -> Result<(), SubscriberError> {
        repository
            .modify(self.id, |subscriber| {
                subscriber.update(self.name.clone());
                Ok(())
            })
            .await
    }
}

pub struct VerifySubscriberEmailAs {
    id: Uuid,
    statuc: SubscriberEmailVerifiationStatus,
}

impl VerifySubscriberEmailAs {
    pub fn new(id: Uuid, status: SubscriberEmailVerifiationStatus) -> Self {
        Self { id, statuc: status }
    }

    async fn execute(&self, repository: impl SubscriberRepository) -> Result<(), SubscriberError> {
        repository
            .modify(self.id, |subscriber| {
                subscriber.verified_email_as(&self.statuc)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use claims::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::LastName;
    use fake::Fake;

    use crate::subscriber::prelude::{
        FakeSubscriberRepository,
        Subscriber,
        SubscriberEvent,
    };

    use super::*;

    async fn create_subscriber(id: Uuid) -> Subscriber {
        let mut subscriber = Subscriber::default();
        subscriber.create(id, SafeEmail().fake(), LastName().fake());
        subscriber
    }

    #[tokio::test]
    async fn create_subscriber_succeeds() {
        // given
        let repository = FakeSubscriberRepository::new();

        // when
        let command = CreateSubscriber::new(Uuid::new_v4(), SafeEmail().fake(), LastName().fake());
        let response = command.execute(repository).await;

        // then
        assert_ok!(response);
    }

    #[tokio::test]
    async fn create_subscriber_twice_with_same_subscriber_id_fails() {
        // given
        let repository = FakeSubscriberRepository::new();
        let id = Uuid::new_v4();
        let command = CreateSubscriber::new(id, SafeEmail().fake(), LastName().fake());
        command.execute(repository.clone()).await.unwrap();

        // when
        let command = CreateSubscriber::new(id, SafeEmail().fake(), LastName().fake());
        let response = command.execute(repository.clone()).await;

        // then
        assert_err!(response);
    }

    #[tokio::test]
    async fn subscriber_email_verification_as_unverified_raises_an_error() {
        // given
        let repository = FakeSubscriberRepository::new();
        let id = Uuid::new_v4();
        let mut subscriber = create_subscriber(id).await;
        repository.save(&mut subscriber).await.unwrap();

        // when
        let command =
            VerifySubscriberEmailAs::new(id, SubscriberEmailVerifiationStatus::Unverified);
        let response = command.execute(repository).await;

        // then
        assert_err!(response);
    }

    #[tokio::test]
    async fn subscriber_email_verification_as_valid_updates_subscriber_email_status_to_valid() {
        // given
        let repository = FakeSubscriberRepository::new();
        let id = Uuid::new_v4();
        let mut subscriber = create_subscriber(id).await;
        repository.save(&mut subscriber).await.unwrap();

        // when
        let command = VerifySubscriberEmailAs::new(id, SubscriberEmailVerifiationStatus::Valid);
        command.execute(repository.clone()).await.unwrap();

        // then
        let events = repository.clone().find_events_by_id(id).await.unwrap();
        assert_eq!(events.len(), 2);
        assert_matches!(
            events.last().unwrap(),
            SubscriberEvent::SubscriberEmailVerifiedAsValid(_)
        );
    }
}
