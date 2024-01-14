use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::{Executor, Pool, Postgres, Row};
use uuid::Uuid;

use crate::domain::subscription::subscriber::error::SubscriberError;
use crate::domain::subscription::subscriber::model::{
    Subscriber, SubscriberEmail, SubscriberName, SubscriberStatus,
};
use crate::domain::subscription::subscriber::repository::SubscriberRepository;

struct SubscriberDataModel {
    id: Uuid,
    email: String,
    name: String,
    status: String,
    subscribed_at: DateTime<Utc>,
}

impl SubscriberDataModel {
    pub fn new(id: Uuid, email: String, name: String, status: String) -> Self {
        Self {
            id,
            email,
            name,
            status,
            subscribed_at: Utc::now(),
        }
    }
}

// TODO: Implement the mapping with ORM later
impl From<&Subscriber> for SubscriberDataModel {
    fn from(subscriber: &Subscriber) -> Self {
        Self::new(
            subscriber.id,
            subscriber.email.as_ref().to_string(),
            subscriber.name.as_ref().to_string(),
            // TODO: Implement elegant enum in domain to string in data model mapping
            match subscriber.status {
                SubscriberStatus::Confirmed => "Confirmed".to_string(),
                SubscriberStatus::Unconfirmed => "Unconfirmed".to_string(),
            },
        )
    }
}

impl From<&PgRow> for SubscriberDataModel {
    fn from(row: &PgRow) -> Self {
        Self {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            status: row.get(3),
            subscribed_at: row.get(4),
        }
    }
}

impl TryFrom<SubscriberDataModel> for Subscriber {
    type Error = SubscriberError;

    fn try_from(data_model: SubscriberDataModel) -> Result<Self, Self::Error> {
        let email = SubscriberEmail::parse(data_model.email)?;
        let name = SubscriberName::parse(data_model.name)?;
        let status = match data_model.status.as_ref() {
            "Confirmed" => Ok(SubscriberStatus::Confirmed),
            "Unconfirmed" => Ok(SubscriberStatus::Unconfirmed),
            _ => Err(SubscriberError::RepositoryOperationFailed(anyhow::anyhow!(
                "Failed to parse as SubscriberStatus",
            ))),
        }?;

        Ok(Self {
            id: data_model.id,
            email,
            name,
            status,
        })
    }
}

#[derive(Clone)]
pub struct SubscriberPostgresRepository {
    pool: Pool<Postgres>,
}

impl SubscriberPostgresRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriberRepository for SubscriberPostgresRepository {
    #[tracing::instrument(name = "Saving subscriber details", skip(self))]
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError> {
        let data_model = SubscriberDataModel::from(subscriber);
        let query = sqlx::query!(
            "INSERT INTO subscribers (id, email, name, subscribed_at, status) VALUES ($1, $2, $3, $4, $5)",
            data_model.id,
            data_model.email,
            data_model.name,
            data_model.subscribed_at,
            data_model.status,
        );

        self.pool
            .acquire()
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .execute(query)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Updating subscriber details", skip(self))]
    async fn update(&self, subscriber: &Subscriber) -> Result<(), SubscriberError> {
        let data_model = SubscriberDataModel::from(subscriber);
        let query = sqlx::query!(
            "UPDATE subscribers SET email = $2, name = $3, status = $4 WHERE id = $1",
            data_model.id,
            data_model.email,
            data_model.name,
            data_model.status,
        );

        self.pool
            .acquire()
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .execute(query)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Searching subscriber details by ID", skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError> {
        let query = sqlx::query!(
            "SELECT id, email, name, status, subscribed_at FROM subscribers WHERE id = $1",
            id
        );

        let optional_data_model = self
            .pool
            .acquire()
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .fetch_optional(query)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(|row| SubscriberDataModel::from(&row));

        match optional_data_model {
            Some(data_model) => Ok(Some(Subscriber::try_from(data_model)?)),
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Searching subscriber details by email", skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError> {
        let query = sqlx::query!(
            "SELECT id, email, name, status, subscribed_at FROM subscribers WHERE email = $1",
            email
        );

        let optional_data_model = self
            .pool
            .acquire()
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .fetch_optional(query)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(|row| SubscriberDataModel::from(&row));

        match optional_data_model {
            Some(data_model) => Ok(Some(Subscriber::try_from(data_model)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::FirstName;
    use fake::Fake;
    use uuid::Uuid;

    use crate::configuration::*;
    use crate::domain::subscription::subscriber::model::Subscriber;
    use crate::domain::subscription::subscriber::repository::SubscriberRepository;
    use crate::infrastructure::subscription::subscriber::*;

    async fn get_repository() -> SubscriberPostgresRepository {
        let configuration = get_configuration().await;

        SubscriberPostgresRepository::new(
            sqlx::Pool::connect(&configuration.database.connection_string_with_database())
                .await
                .unwrap(),
        )
    }

    fn generate_subscriber() -> Subscriber {
        let id = Uuid::new_v4();
        let email = SafeEmail().fake();
        let name = FirstName().fake();

        Subscriber::new(id, email, name).unwrap()
    }

    #[tokio::test]
    async fn fetching_by_id_after_saving_via_repository_makes_the_same_subscriber() {
        // given
        let repository = get_repository().await;
        let subscriber = generate_subscriber();

        // when
        repository.save(&subscriber).await.unwrap();

        // then
        let saved_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(saved_subscriber.id, subscriber.id);
        assert_eq!(saved_subscriber.email, subscriber.email);
        assert_eq!(saved_subscriber.name, subscriber.name);
        assert_eq!(saved_subscriber.status, subscriber.status);
    }

    #[tokio::test]
    async fn fetching_not_existing_subscriber_should_return_option_null() {
        // given
        let repository = get_repository().await;
        let subscriber = generate_subscriber();

        // when
        // do nothing, not saved subscriber

        // then
        let not_existing_subscriber = repository.find_by_id(subscriber.id).await.unwrap();
        assert!(not_existing_subscriber.is_none());
    }
}
