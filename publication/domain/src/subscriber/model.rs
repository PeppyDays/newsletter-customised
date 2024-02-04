use uuid::Uuid;

use crate::subscriber::error::SubscriberError;

#[derive(Debug)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: SubscriberEmail,
    pub name: String,
}

impl Subscriber {
    pub fn new(id: Uuid, email: SubscriberEmail, name: String) -> Self {
        Self { id, email, name }
    }
}

#[derive(Debug)]
pub struct SubscriberEmail {
    pub address: String,
    pub verification_status: SubscriberEmailVerifiationStatus,
}

impl SubscriberEmail {
    pub fn new(address: String) -> Self {
        Self {
            address,
            verification_status: SubscriberEmailVerifiationStatus::Unverified,
        }
    }

    pub fn verify_as(
        &mut self,
        status: SubscriberEmailVerifiationStatus,
    ) -> Result<(), SubscriberError> {
        match status {
            SubscriberEmailVerifiationStatus::Unverified => {
                Err(SubscriberError::InvalidSubscriberEmailVerificationStatus)
            }
            _ => {
                self.verification_status = status;
                Ok(())
            }
        }
    }

    pub fn initialize_verification(&mut self) {
        self.verification_status = SubscriberEmailVerifiationStatus::Unverified;
    }
}

#[derive(Debug, PartialEq)]
pub enum SubscriberEmailVerifiationStatus {
    Unverified,
    Valid,
    Invalid,
}

#[cfg(test)]
mod tests {
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::LastName;
    use fake::Fake;

    use super::*;

    fn get_subscriber() -> Subscriber {
        let id = Uuid::new_v4();
        let email = SubscriberEmail::new(SafeEmail().fake());
        let name = LastName().fake();

        Subscriber::new(id, email, name)
    }

    #[test]
    fn verified_as_unverified_status_is_rejected() {
        // given
        let mut subscriber = get_subscriber();

        // when
        let response = subscriber
            .email
            .verify_as(SubscriberEmailVerifiationStatus::Unverified);

        // then
        assert_err!(response);
    }

    #[test]
    fn verification_initialisation_modify_status_as_unverified() {
        // given
        let mut subscriber = get_subscriber();

        // when
        subscriber.email.initialize_verification();

        // then
        assert_eq!(
            subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Unverified
        );
    }
}
