use uuid::Uuid;

use crate::subscriber::error::SubscriberError;

#[derive(Debug)]
pub struct Subscriber {
    pub id: Uuid,
    pub email: SubscriberEmail,
    pub name: SubscriberName,
    pub status: SubscriberStatus,
}

impl Subscriber {
    pub fn new(id: Uuid, email: SubscriberEmail, name: SubscriberName) -> Self {
        Self {
            id,
            email,
            name,
            status: SubscriberStatus::Unconfirmed,
        }
    }

    // TODO: Remove this method
    pub fn register(id: Uuid, email: String, name: String) -> Result<Self, SubscriberError> {
        let name = SubscriberName::parse(name)?;
        let email = SubscriberEmail::parse(email)?;

        Ok(Self {
            id,
            email,
            name,
            status: SubscriberStatus::Unconfirmed,
        })
    }

    pub fn confirm(&mut self) {
        self.status = SubscriberStatus::Confirmed;
    }
}

#[derive(Debug, PartialEq, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub enum SubscriberStatus {
    Confirmed,
    Unconfirmed,
    Unknown,
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

#[cfg(test)]
mod tests {
    use claims::{
        assert_err,
        assert_ok,
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use crate::subscriber::model::{
        SubscriberEmail,
        SubscriberName,
    };

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0).is_ok()
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
}
