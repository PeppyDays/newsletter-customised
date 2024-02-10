use uuid::Uuid;

use crate::subscriber::model::entities::{
    Subscriber,
    SubscriberEmailVerifiationStatus,
};

#[derive(Clone, Debug)]
pub enum SubscriberEvent {
    SubscriberCreated(SubscriberCreated),
    SubscriberUpdated(SubscriberUpdated),
    SubscriberEmailVerifiedAsValid(SubscriberEmailVerifiedAsValid),
    SubscriberEmailVerifiedAsInvalid(SubscriberEmailVerifiedAsInvalid),
}

#[derive(Clone, Debug)]
pub struct SubscriberCreated {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

impl SubscriberCreated {
    pub fn new(id: Uuid, email: String, name: String) -> Self {
        Self { id, email, name }
    }

    pub fn apply(self, subscriber: &mut Subscriber) {
        subscriber.id = self.id;
        subscriber.email.address = self.email;
        subscriber.name = self.name;
    }
}

#[derive(Clone, Debug)]
pub struct SubscriberUpdated {
    pub name: String,
}

impl SubscriberUpdated {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn apply(self, subscriber: &mut Subscriber) {
        subscriber.name = self.name;
    }
}

#[derive(Clone, Debug)]
pub struct SubscriberEmailVerifiedAsValid {}

impl SubscriberEmailVerifiedAsValid {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply(self, subscriber: &mut Subscriber) {
        subscriber.email.verification_status = SubscriberEmailVerifiationStatus::Valid;
    }
}

#[derive(Clone, Debug)]
pub struct SubscriberEmailVerifiedAsInvalid {}

impl SubscriberEmailVerifiedAsInvalid {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {}
    }

    pub fn apply(self, subscriber: &mut Subscriber) {
        subscriber.email.verification_status = SubscriberEmailVerifiationStatus::Invalid;
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::LastName;
    use fake::Fake;

    use crate::subscriber::prelude::SubscriberEmailVerifiationStatus;

    use super::*;

    #[tokio::test]
    async fn applying_subscriber_created_event_sets_subscriber_status() {
        // given
        let id = Uuid::new_v4();
        let email: String = SafeEmail().fake();
        let name: String = LastName().fake();
        let event = SubscriberEvent::SubscriberCreated(SubscriberCreated::new(
            id,
            email.clone(),
            name.clone(),
        ));

        // when
        let mut subscriber = Subscriber::default();
        subscriber.apply(event);

        // then
        assert_eq!(subscriber.id, id);
        assert_eq!(subscriber.name, name);
        assert_eq!(subscriber.email.address, email);
        assert_eq!(
            subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Unverified
        );
    }

    #[tokio::test]
    async fn applying_subscriber_updated_event_sets_name() {
        // given
        let name: String = LastName().fake();
        let event = SubscriberEvent::SubscriberUpdated(SubscriberUpdated::new(name.clone()));

        // when
        let mut subscriber = Subscriber::default();
        subscriber.apply(event);

        // then
        assert_eq!(subscriber.name, name);
    }

    #[tokio::test]
    async fn applying_subscriber_email_verified_as_valid_event_sets_status_to_valid() {
        // given
        let event =
            SubscriberEvent::SubscriberEmailVerifiedAsValid(SubscriberEmailVerifiedAsValid::new());

        // when
        let mut subscriber = Subscriber::default();
        subscriber.apply(event);

        // then
        assert_eq!(
            subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Valid
        );
    }

    #[tokio::test]
    async fn applying_subscriber_email_verified_as_invalid_event_sets_status_to_invalid() {
        // given
        let event = SubscriberEvent::SubscriberEmailVerifiedAsInvalid(
            SubscriberEmailVerifiedAsInvalid::new(),
        );

        // when
        let mut subscriber = Subscriber::default();
        subscriber.apply(event);

        // then
        assert_eq!(
            subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Invalid
        );
    }
}
