use uuid::Uuid;

use crate::subscriber::model::error::SubscriberError;
use crate::subscriber::model::events::{
    SubscriberCreated,
    SubscriberEmailVerifiedAsInvalid,
    SubscriberEmailVerifiedAsValid,
    SubscriberEvent,
    SubscriberUpdated,
};

#[derive(Default, Clone, Debug)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: SubscriberEmail,
    pub name: String,
    pub pending_events: Vec<SubscriberEvent>,
}

impl Subscriber {
    pub fn create(&mut self, id: Uuid, email: String, name: String) {
        let event = SubscriberEvent::SubscriberCreated(SubscriberCreated::new(id, email, name));
        self.apply(event);
    }

    pub fn update(&mut self, name: String) {
        let event = SubscriberEvent::SubscriberUpdated(SubscriberUpdated::new(name));
        self.apply(event);
    }

    pub fn verified_email_as(
        &mut self,
        status: &SubscriberEmailVerifiationStatus,
    ) -> Result<(), SubscriberError> {
        match status {
            SubscriberEmailVerifiationStatus::Unverified => {
                Err(SubscriberError::InvalidSubscriberEmailVerificationStatus)
            }
            SubscriberEmailVerifiationStatus::Valid => {
                let event = SubscriberEvent::SubscriberEmailVerifiedAsValid(
                    SubscriberEmailVerifiedAsValid::new(),
                );
                self.apply(event);
                Ok(())
            }
            SubscriberEmailVerifiationStatus::Invalid => {
                let event = SubscriberEvent::SubscriberEmailVerifiedAsInvalid(
                    SubscriberEmailVerifiedAsInvalid::new(),
                );
                self.apply(event);
                Ok(())
            }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct SubscriberEmail {
    pub address: String,
    pub verification_status: SubscriberEmailVerifiationStatus,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum SubscriberEmailVerifiationStatus {
    #[default]
    Unverified,
    Valid,
    Invalid,
}

impl Subscriber {
    pub fn apply(&mut self, event: SubscriberEvent) {
        self.pending_events.push(event.clone());

        match event {
            SubscriberEvent::SubscriberCreated(event) => {
                event.apply(self);
            }
            SubscriberEvent::SubscriberUpdated(event) => {
                event.apply(self);
            }
            SubscriberEvent::SubscriberEmailVerifiedAsValid(event) => {
                event.apply(self);
            }
            SubscriberEvent::SubscriberEmailVerifiedAsInvalid(event) => {
                event.apply(self);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::LastName;
    use fake::Fake;

    use super::*;

    fn get_subscriber() -> Subscriber {
        Subscriber {
            id: Uuid::new_v4(),
            email: SubscriberEmail {
                address: SafeEmail().fake(),
                verification_status: SubscriberEmailVerifiationStatus::Unverified,
            },
            name: LastName().fake(),
            pending_events: vec![],
        }
    }

    #[test]
    fn verified_as_unverified_status_is_rejected() {
        // given
        let mut subscriber = get_subscriber();

        // when
        let response = subscriber.verified_email_as(&SubscriberEmailVerifiationStatus::Unverified);

        // then
        assert_err!(response);
    }
}
