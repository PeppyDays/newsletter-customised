use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, Executor, Pool, Postgres, Row};
use uuid::Uuid;

use crate::domain::subscriber::{Subscriber, SubscriberError, SubscriberRepository};

struct SubscriberDataModel {
    id: Uuid,
    email: String,
    name: String,
    subscribed_at: DateTime<Utc>,
}

impl SubscriberDataModel {
    pub fn new(id: Uuid, email: String, name: String) -> Self {
        Self {
            id,
            email,
            name,
            subscribed_at: Utc::now(),
        }
    }
}

// TODO: Implement the mapping with ORM later
impl From<&Subscriber> for SubscriberDataModel {
    fn from(subscriber: &Subscriber) -> Self {
        Self::new(
            subscriber.id,
            subscriber.email.clone(),
            subscriber.name.clone(),
        )
    }
}

impl From<&PgRow> for SubscriberDataModel {
    fn from(row: &PgRow) -> Self {
        Self {
            id: row.get(0),
            email: row.get(1),
            name: row.get(2),
            subscribed_at: row.get(3),
        }
    }
}

impl From<SubscriberDataModel> for Subscriber {
    fn from(subscriber_data_model: SubscriberDataModel) -> Self {
        Self::new(
            subscriber_data_model.id,
            subscriber_data_model.email,
            subscriber_data_model.name,
        )
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
            "INSERT INTO subscribers (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)",
            data_model.id,
            data_model.email,
            data_model.name,
            data_model.subscribed_at,
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
            "SELECT id, email, name, subscribed_at FROM subscribers WHERE id = $1",
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
            Some(data_model) => Ok(Some(Subscriber::from(data_model))),
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Searching subscriber details by email", skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError> {
        let query = sqlx::query!(
            "SELECT id, email, name, subscribed_at FROM subscribers WHERE email = $1",
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
            Some(data_model) => Ok(Some(Subscriber::from(data_model))),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::FirstName;
    use fake::Fake;

    use crate::configuration::*;
    use crate::domain::subscriber::*;
    use crate::infrastructure::repositories::subscriber::*;

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

        Subscriber::new(id, email, name)
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
