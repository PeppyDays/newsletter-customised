use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use uuid::Uuid;

use crate::domain::subscription::subscriber::error::SubscriberError;
use crate::domain::subscription::subscriber::model::{
    Subscriber, SubscriberEmail, SubscriberName, SubscriberStatus,
};
use crate::domain::subscription::subscriber::repository::SubscriberRepository;

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
            status: ActiveValue::Set(match subscriber.status {
                SubscriberStatus::Confirmed => "Confirmed".to_string(),
                SubscriberStatus::Unconfirmed => "Unconfirmed".to_string(),
                SubscriberStatus::Unknown => "Unknown".to_string(),
            }),
        }
    }
}

impl From<Model> for Subscriber {
    fn from(data_model: Model) -> Self {
        Self {
            id: data_model.id,
            email: SubscriberEmail::parse(data_model.email).unwrap(),
            name: SubscriberName::parse(data_model.name).unwrap(),
            status: match data_model.status.as_ref() {
                "Confirmed" => SubscriberStatus::Confirmed,
                "Unconfirmed" => SubscriberStatus::Unconfirmed,
                _ => SubscriberStatus::Unknown,
            },
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
        data_model
            .insert(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Updating subscriber details", skip(self))]
    async fn update(&self, subscriber: &Subscriber) -> Result<(), SubscriberError> {
        let data_model = ActiveModel::from(subscriber);
        data_model
            .update(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Searching subscriber details by ID", skip(self))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Subscriber>, SubscriberError> {
        let data_model = Entity::find()
            .filter(Column::Id.eq(id))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        match data_model {
            Some(data_model) => Ok(Some(Subscriber::from(data_model))),
            None => Ok(None),
        }
    }

    #[tracing::instrument(name = "Searching subscriber details by email", skip(self))]
    async fn find_by_email(&self, email: &str) -> Result<Option<Subscriber>, SubscriberError> {
        let data_model = Entity::find()
            .filter(Column::Email.eq(email))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriberError::RepositoryOperationFailed(error.into()))?;

        match data_model {
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
    use sea_orm::Database;
    use uuid::Uuid;

    use crate::configuration::*;
    use crate::domain::subscription::subscriber::model::Subscriber;
    use crate::domain::subscription::subscriber::repository::SubscriberRepository;

    use super::*;

    async fn get_repository() -> SubscriberSeaOrmRepository {
        let configuration = get_configuration().await;

        SubscriberSeaOrmRepository::new(
            Database::connect(&configuration.database.connection_string_without_database())
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
