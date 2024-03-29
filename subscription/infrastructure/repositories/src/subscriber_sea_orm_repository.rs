use std::future::Future;

use anyhow::Context;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveValue,
    TransactionTrait,
};
use uuid::Uuid;

use domain::prelude::{
    Subscriber,
    SubscriberEmail,
    SubscriberError,
    SubscriberName,
    SubscriberRepository,
    SubscriberStatus,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text", unique)]
    pub email: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    #[sea_orm(column_type = "Text")]
    pub status: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<&Subscriber> for ActiveModel {
    fn from(subscriber: &Subscriber) -> Self {
        Self {
            id: ActiveValue::Set(subscriber.id),
            email: ActiveValue::Set(subscriber.email.as_ref().to_string()),
            name: ActiveValue::Set(subscriber.name.as_ref().to_string()),
            status: ActiveValue::Set(subscriber.status.as_ref().to_string()),
        }
    }
}

impl From<Model> for Subscriber {
    fn from(data_model: Model) -> Self {
        Self {
            id: data_model.id,
            email: SubscriberEmail::parse(data_model.email).unwrap(),
            name: SubscriberName::parse(data_model.name).unwrap(),
            status: SubscriberStatus::parse(data_model.status).unwrap(),
        }
    }
}

#[derive(Clone)]
pub struct SubscriberSeaOrmRepository {
    pool: DatabaseConnection,
}

impl SubscriberSeaOrmRepository {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriberRepository for SubscriberSeaOrmRepository {
    #[tracing::instrument(name = "Saving subscriber details", skip(self))]
    async fn save(&self, subscriber: &Subscriber) -> Result<(), SubscriberError> {
        let data_model = ActiveModel::from(subscriber);

        Entity::insert(data_model)
            .on_conflict(
                OnConflict::column(Column::Id)
                    .update_columns([Column::Email, Column::Name, Column::Status])
                    .to_owned(),
            )
            .exec(&self.pool)
            .await
            .map_err(|error| {
                if error
                    .to_string()
                    .contains("duplicate key value violates unique constraint")
                {
                    SubscriberError::InvalidSubscriberEmail
                } else {
                    SubscriberError::RepositoryOperationFailed(error.into())
                }
            })?;

        Ok(())
    }

    #[tracing::instrument(
        name = "Modifying subscriber with modifier and saving the modified subscriber",
        skip(self, modifier)
    )]
    async fn modify<F, Fut>(&self, id: Uuid, modifier: F) -> Result<(), SubscriberError>
    where
        F: Fn(Subscriber) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Subscriber, SubscriberError>> + Send + 'static,
    {
        let transaction = self
            .pool
            .begin()
            .await
            .context("Failed to start a transaction")
            .map_err(SubscriberError::RepositoryOperationFailed)?;

        let subscriber = Entity::find()
            .filter(Column::Id.eq(id))
            .one(&transaction)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(Subscriber::from)
            .ok_or(SubscriberError::SubscriberNotFound(id))?;

        let subscriber = modifier(subscriber).await?;
        self.save(&subscriber).await?;

        transaction
            .commit()
            .await
            .context("Failed to commit a transaction")
            .map_err(SubscriberError::RepositoryOperationFailed)?;

        Ok(())
    }

    #[tracing::instrument(name = "Searching subscriber details by ID", skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError> {
        Ok(Entity::find()
            .filter(Column::Id.eq(id))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(Subscriber::from))
    }

    #[tracing::instrument(name = "Searching subscriber details by email", skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError> {
        Ok(Entity::find()
            .filter(Column::Email.eq(email))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(Subscriber::from))
    }

    // TODO: Modify this to streaming rows
    #[tracing::instrument(name = "Searching subscriber details by status", skip(self))]
    async fn find_by_status(
        &self,
        status: SubscriberStatus,
    ) -> Result<Vec<Subscriber>, SubscriberError> {
        Ok(Entity::find()
            .filter(Column::Status.eq(status.as_ref()))
            .all(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .into_iter()
            .map(Subscriber::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::FirstName;
    use fake::Fake;
    use sea_orm::ConnectionTrait;
    use uuid::Uuid;

    use super::*;

    async fn get_repository(isolated: bool) -> SubscriberSeaOrmRepository {
        let url_without_db = "postgres://subscription:welcome@localhost:15432";

        if !isolated {
            let url = format!("{}/{}", &url_without_db, "subscription");
            let pool = sea_orm::Database::connect(&url).await.unwrap();

            return SubscriberSeaOrmRepository::new(pool);
        }

        let db = format!("{}_{}", "test", 10.fake::<String>());
        let url = format!("{}/{}", url_without_db, db);

        // https://www.sea-ql.org/sea-orm-tutorial/ch02-02-connect-to-database.html
        let connection = sea_orm::Database::connect(url_without_db).await.unwrap();
        connection
            .execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("DROP DATABASE IF EXISTS \"{}\";", &db),
            ))
            .await
            .unwrap();
        connection
            .execute(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                format!("CREATE DATABASE \"{}\";", &db),
            ))
            .await
            .unwrap();

        let pool = sea_orm::Database::connect(&url).await.unwrap();
        sqlx::migrate!("./migrations")
            .run(pool.get_postgres_connection_pool())
            .await
            .unwrap();

        SubscriberSeaOrmRepository::new(pool)
    }

    fn generate_subscriber() -> Subscriber {
        let id = Uuid::new_v4();
        let email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let name = SubscriberName::parse(FirstName().fake()).unwrap();

        Subscriber::new(id, email, name)
    }

    #[tokio::test]
    async fn fetching_by_id_after_saving_via_repository_makes_the_same_subscriber() {
        // given
        let repository = get_repository(false).await;
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
        let repository = get_repository(false).await;
        let subscriber = generate_subscriber();

        // when
        // do nothing, not saved subscriber

        // then
        let not_existing_subscriber = repository.find_by_id(subscriber.id).await.unwrap();
        assert!(not_existing_subscriber.is_none());
    }

    #[tokio::test]
    async fn saving_entity_two_times_will_update_original_entity() {
        // given
        let repository = get_repository(false).await;
        let mut subscriber = generate_subscriber();
        assert_eq!(subscriber.status, SubscriberStatus::Unconfirmed);

        repository.save(&subscriber).await.unwrap();

        // when
        subscriber.status = SubscriberStatus::Confirmed;
        repository.save(&subscriber).await.unwrap();

        // then
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(persisted_subscriber.status, SubscriberStatus::Confirmed);
    }

    #[tokio::test]
    async fn searching_by_status_with_confirmed_returns_only_confirmed_subscribers() {
        // given
        let repository = get_repository(true).await;
        let unconfirmed_subscriber_1 = generate_subscriber();
        let unconfirmed_subscriber_2 = generate_subscriber();
        let mut confirmed_subscriber = generate_subscriber();
        confirmed_subscriber.status = SubscriberStatus::Confirmed;

        repository.save(&unconfirmed_subscriber_1).await.unwrap();
        repository.save(&unconfirmed_subscriber_2).await.unwrap();
        repository.save(&confirmed_subscriber).await.unwrap();

        // when
        let response = repository
            .find_by_status(SubscriberStatus::Confirmed)
            .await
            .unwrap();

        // then
        assert_eq!(response.len(), 1);

        let persisted_subscriber = response.first().unwrap();
        assert_eq!(persisted_subscriber.id, confirmed_subscriber.id);
    }

    #[tokio::test]
    async fn saving_second_subscriber_with_existing_email_returns_invalid_subscriber_email_error() {
        // given
        let repository = get_repository(false).await;
        let mut confirmed_subscriber = generate_subscriber();
        confirmed_subscriber.status = SubscriberStatus::Confirmed;

        repository.save(&confirmed_subscriber).await.unwrap();

        // when
        let mut new_subscriber_with_same_email = generate_subscriber();
        new_subscriber_with_same_email.email = confirmed_subscriber.email;

        let response = repository.save(&new_subscriber_with_same_email).await;

        // then
        assert!(response.is_err());
        assert!(matches!(
            response.unwrap_err(),
            SubscriberError::InvalidSubscriberEmail
        ));
    }

    #[tokio::test]
    async fn modifying_subscriber_status_to_confirmed_succeeds_when_no_errors_from_repository() {
        // given
        let repository = get_repository(true).await;
        let subscriber = generate_subscriber();
        repository.save(&subscriber).await.unwrap();

        // when
        repository
            .modify(subscriber.id, |mut subscriber| async {
                subscriber.confirm();
                Ok(subscriber)
            })
            .await
            .unwrap();

        // then
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(persisted_subscriber.status, SubscriberStatus::Confirmed);
    }

    #[tokio::test]
    async fn modifying_subscriber_ensures_atomic_operation_despite_of_repository_error() {
        // given
        let repository = get_repository(true).await;
        let subscriber = generate_subscriber();
        repository.save(&subscriber).await.unwrap();

        // when
        let response = repository
            .modify(subscriber.id, |mut subscriber| async move {
                subscriber.confirm();
                Err(SubscriberError::RepositoryOperationFailed(anyhow::anyhow!(
                    "Some errors"
                )))
            })
            .await;

        // then
        assert!(response.is_err());
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(persisted_subscriber.status, SubscriberStatus::Unconfirmed);
    }
}
