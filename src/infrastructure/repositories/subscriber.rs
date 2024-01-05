use chrono::{DateTime, Utc};
use sqlx::{postgres::PgRow, Pool, Postgres, Row};
use uuid::Uuid;

use crate::domain::subscriber::{Subscriber, SubscriberRepository};

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

impl SubscriberPostgresRepository {
    const SAVE_QUERY: &'static str =
        "INSERT INTO subscribers (id, email, name, subscribed_at) VALUES ($1, $2, $3, $4)";
    const FIND_BY_ID_QUERY: &'static str =
        "SELECT id, email, name, subscribed_at FROM subscribers WHERE id = $1";
    const FIND_BY_EMAIL_QUERY: &'static str =
        "SELECT id, email, name, subscribed_at FROM subscribers WHERE email = $1";
}

#[async_trait::async_trait]
impl SubscriberRepository for SubscriberPostgresRepository {
    async fn save(&self, subscriber: &Subscriber) -> Result<(), String> {
        let data_model = SubscriberDataModel::from(subscriber);

        sqlx::query(SubscriberPostgresRepository::SAVE_QUERY)
            .bind(data_model.id)
            .bind(data_model.email)
            .bind(data_model.name)
            .bind(data_model.subscribed_at)
            .execute(&self.pool)
            .await
            .unwrap();

        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, String> {
        Ok(sqlx::query(SubscriberPostgresRepository::FIND_BY_ID_QUERY)
            .bind(id)
            .map(|row: PgRow| SubscriberDataModel::from(&row))
            .fetch_optional(&self.pool)
            .await
            .unwrap()
            .map(Subscriber::from))
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, String> {
        Ok(
            sqlx::query(SubscriberPostgresRepository::FIND_BY_EMAIL_QUERY)
                .bind(email)
                .map(|row: PgRow| SubscriberDataModel::from(&row))
                .fetch_optional(&self.pool)
                .await
                .unwrap()
                .map(Subscriber::from),
        )
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
            sqlx::Pool::connect(&configuration.database.connection_string())
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
    async fn fetching_after_saving_via_repository_makes_the_same_subscriber() {
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
