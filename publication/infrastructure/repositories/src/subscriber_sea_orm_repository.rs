use anyhow::Context;
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, TransactionTrait};
use uuid::Uuid;

use domain::prelude::{
    Subscriber, SubscriberEmail, SubscriberEmailVerifiationStatus, SubscriberError,
    SubscriberRepository,
};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "subscribers")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    #[sea_orm(column_type = "Text", unique)]
    pub email_address: String,
    pub email_verification_status: EmailVerificationStatus,
    #[sea_orm(column_type = "Text")]
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(
    rs_type = "String",
    db_type = "Enum",
    enum_name = "EMAIL_VERIFICATION_STATUS"
)]
pub enum EmailVerificationStatus {
    #[sea_orm(string_value = "Invalid")]
    Invalid,
    #[sea_orm(string_value = "Unverified")]
    Unverified,
    #[sea_orm(string_value = "Valid")]
    Valid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<&SubscriberEmailVerifiationStatus> for EmailVerificationStatus {
    fn from(status: &SubscriberEmailVerifiationStatus) -> Self {
        match status {
            SubscriberEmailVerifiationStatus::Unverified => EmailVerificationStatus::Unverified,
            SubscriberEmailVerifiationStatus::Valid => EmailVerificationStatus::Valid,
            SubscriberEmailVerifiationStatus::Invalid => EmailVerificationStatus::Invalid,
        }
    }
}

impl From<EmailVerificationStatus> for SubscriberEmailVerifiationStatus {
    fn from(status: EmailVerificationStatus) -> Self {
        match status {
            EmailVerificationStatus::Unverified => SubscriberEmailVerifiationStatus::Unverified,
            EmailVerificationStatus::Valid => SubscriberEmailVerifiationStatus::Valid,
            EmailVerificationStatus::Invalid => SubscriberEmailVerifiationStatus::Invalid,
        }
    }
}

impl From<&Subscriber> for ActiveModel {
    fn from(subscriber: &Subscriber) -> Self {
        Self {
            id: ActiveValue::Set(subscriber.id),
            email_address: ActiveValue::Set(subscriber.email.address.clone()),
            email_verification_status: ActiveValue::Set(EmailVerificationStatus::from(
                &subscriber.email.verification_status,
            )),
            name: ActiveValue::Set(subscriber.name.clone()),
        }
    }
}

impl From<Model> for Subscriber {
    fn from(data_model: Model) -> Self {
        Self {
            id: data_model.id,
            email: SubscriberEmail {
                address: data_model.email_address,
                verification_status: SubscriberEmailVerifiationStatus::from(
                    data_model.email_verification_status,
                ),
            },
            name: data_model.name,
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

impl SubscriberSeaOrmRepository {
    #[tracing::instrument(name = "Searching subscriber details by ID", skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError> {
        Ok(Entity::find()
            .filter(Column::Id.eq(id))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?
            .map(Subscriber::from))
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
                    .update_columns([
                        Column::EmailAddress,
                        Column::EmailVerificationStatus,
                        Column::Name,
                    ])
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

    #[tracing::instrument(name = "Modifying subscriber details", skip(self, modifier))]
    async fn modify(
        &self,
        id: Uuid,
        modifier: fn(Subscriber) -> Result<Subscriber, SubscriberError>,
    ) -> Result<(), SubscriberError> {
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

        let subscriber = modifier(subscriber)?;
        self.save(&subscriber).await?;

        transaction
            .commit()
            .await
            .context("Failed to commit a transaction")
            .map_err(SubscriberError::RepositoryOperationFailed)?;

        Ok(())
    }

    #[tracing::instrument(name = "Fetching all subscribers", skip(self))]
    async fn find_all(&self) -> Result<Vec<Subscriber>, SubscriberError> {
        Ok(Entity::find()
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
        let url_without_db = "postgres://publication:welcome@localhost:25432";

        if !isolated {
            let url = format!("{}/{}", &url_without_db, "publication");
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
        let email = SubscriberEmail::new(SafeEmail().fake());
        let name = FirstName().fake();

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
        assert_eq!(saved_subscriber.email.address, subscriber.email.address);
        assert_eq!(
            saved_subscriber.email.verification_status,
            subscriber.email.verification_status,
        );
        assert_eq!(saved_subscriber.name, subscriber.name);
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

        repository.save(&subscriber).await.unwrap();

        // when
        subscriber
            .email
            .verify_as(SubscriberEmailVerifiationStatus::Valid)
            .unwrap();
        repository.save(&subscriber).await.unwrap();

        // then
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(
            persisted_subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Valid
        );
    }

    #[tokio::test]
    async fn modifying_subscriber_name_succeeds_when_no_errors_from_repository() {
        // given
        let repository = get_repository(true).await;
        let subscriber = generate_subscriber();
        repository.save(&subscriber).await.unwrap();

        // when
        repository
            .modify(subscriber.id, |mut subscriber| {
                subscriber.name = "New name".to_string();
                Ok(subscriber)
            })
            .await
            .unwrap();

        // then
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(persisted_subscriber.name, "New name");
    }

    #[tokio::test]
    async fn modifying_subscriber_ensures_atomic_operation_despite_of_repository_error() {
        // given
        let repository = get_repository(true).await;
        let subscriber = generate_subscriber();
        repository.save(&subscriber).await.unwrap();

        // when
        let response = repository
            .modify(subscriber.id, |mut _subscriber| {
                Err(SubscriberError::RepositoryOperationFailed(anyhow::anyhow!(
                    "Some errors"
                )))
            })
            .await;

        // then
        assert!(response.is_err());
        let persisted_subscriber = repository.find_by_id(subscriber.id).await.unwrap().unwrap();
        assert_eq!(
            persisted_subscriber.email.verification_status,
            SubscriberEmailVerifiationStatus::Unverified,
        );
    }
}
