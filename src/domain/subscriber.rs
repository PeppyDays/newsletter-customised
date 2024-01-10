use uuid::Uuid;

#[derive(Debug)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: String,
    pub name: SubscriberName,
}

impl Subscriber {
    pub fn new(id: Uuid, email: String, name: String) -> Result<Self, SubscriberError> {
        let name = SubscriberName::parse(name)?;
        Ok(Self { id, email, name })
    }
}

#[derive(Debug, PartialEq)]
pub struct SubscriberName(String);

impl SubscriberName {
    const NAME_MIN_LENGTH: usize = 3;
    const NAME_MAX_LENGTH: usize = 256;
    const NAME_FORBIDDEN_CHARACTERS: [char; 10] =
        ['<', '>', '"', '`', '\'', '%', ';', '\\', '{', '}'];

    pub fn parse(s: String) -> Result<Self, SubscriberError> {
        let is_empty_or_whitespace_only = s.trim().is_empty();
        let is_too_short = s.len() < Self::NAME_MIN_LENGTH;
        let is_too_long = s.len() > Self::NAME_MAX_LENGTH;
        let contains_forbidden_characters = s
            .chars()
            .any(|c| Self::NAME_FORBIDDEN_CHARACTERS.contains(&c));

        if is_empty_or_whitespace_only
            || is_too_short
            || is_too_long
            || contains_forbidden_characters
        {
            Err(SubscriberError::InvalidSubscriberName)
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, PartialEq)]
pub struct SubscriberEmail(String);

impl SubscriberEmail {
    pub fn parse(s: String) -> Result<Self, SubscriberError> {
        if validator::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(SubscriberError::InvalidSubscriberEmail)
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[async_trait::async_trait]
pub trait SubscriberRepository: Send + Sync {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError>;
}

#[derive(thiserror::Error, Debug)]
pub enum SubscriberError {
    #[error("Subscriber's name is invalid")]
    InvalidSubscriberName,

    #[error("Subscriber's email is invalid")]
    InvalidSubscriberEmail,

    #[error("Failed to operate on repository")]
    RepositoryOperationFailed(#[source] anyhow::Error),

    #[error("Failed unexpectedly")]
    Unexpected(#[source] anyhow::Error),
}

#[cfg(test)]
mod tests {
    use claims::{assert_err, assert_ok};
    use fake::{faker::internet::en::SafeEmail, Fake};

    use crate::domain::subscriber::{SubscriberEmail, SubscriberName};

    #[test]
    fn short_names_are_rejected() {
        let name = "a".repeat(SubscriberName::NAME_MIN_LENGTH - 1);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn long_names_are_rejected() {
        let name = "a".repeat(SubscriberName::NAME_MAX_LENGTH + 1);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_having_length_boundary_is_accepted() {
        let name_1 = "a".repeat(SubscriberName::NAME_MIN_LENGTH);
        let name_2 = "a".repeat(SubscriberName::NAME_MAX_LENGTH);

        assert_ok!(SubscriberName::parse(name_1));
        assert_ok!(SubscriberName::parse(name_2));
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = "     ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn empty_string_names_is_rejected() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Arine You".to_string();
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn valid_emails_are_parsed_successfully() {
        let email = SafeEmail().fake();
        claims::assert_ok!(SubscriberEmail::parse(email));
    }

    #[test]
    fn empty_string_emails_are_rejected() {
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn emails_missing_at_symbol_are_rejected() {
        let email = "email.example.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn emails_missing_subject_is_rejected() {
        let email = "@gmail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
