use domain::prelude::{SubscriptionToken, SubscriptionTokenError, SubscriptionTokenRepository};
use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "subscription_tokens")]
pub struct Model {
    #[sea_orm(column_type = "Text", primary_key)]
    pub token: String,
    pub subscriber_id: Uuid,
    pub issued_at: DateTimeWithTimeZone,
    pub expired_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<&SubscriptionToken> for ActiveModel {
    fn from(subscription_token: &SubscriptionToken) -> Self {
        ActiveModel {
            token: ActiveValue::Set(subscription_token.token.clone()),
            subscriber_id: ActiveValue::Set(subscription_token.subscriber_id),
            issued_at: ActiveValue::Set(subscription_token.issued_at.into()),
            expired_at: ActiveValue::Set(subscription_token.expired_at.into()),
        }
    }
}

impl From<Model> for SubscriptionToken {
    fn from(model: Model) -> Self {
        Self {
            token: model.token,
            subscriber_id: model.subscriber_id,
            issued_at: model.issued_at.into(),
            expired_at: model.expired_at.into(),
        }
    }
}

#[derive(Clone)]
pub struct SubscriptionTokenSeaOrmRepository {
    pool: DatabaseConnection,
}

impl SubscriptionTokenSeaOrmRepository {
    pub fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionTokenRepository for SubscriptionTokenSeaOrmRepository {
    #[tracing::instrument(name = "Saving subscription token details", skip(self))]
    async fn save(
        &self,
        subscription_token: &SubscriptionToken,
    ) -> Result<(), SubscriptionTokenError> {
        let data_model = ActiveModel::from(subscription_token);
        data_model
            .insert(&self.pool)
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Searching subscription token details by token", skip(self))]
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SubscriptionToken>, SubscriptionTokenError> {
        Ok(Entity::find()
            .filter(Column::Token.eq(token))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?
            .map(SubscriptionToken::from))
    }

    #[tracing::instrument(
        name = "Searching subscription token details by subscriber id",
        skip(self)
    )]
    async fn find_by_subscriber_id(
        &self,
        subscriber_id: Uuid,
    ) -> Result<Option<SubscriptionToken>, SubscriptionTokenError> {
        Ok(Entity::find()
            .filter(Column::SubscriberId.eq(subscriber_id))
            .one(&self.pool)
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?
            .map(SubscriptionToken::from))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use fake::Fake;

    use super::*;

    async fn get_repository(isolated: bool) -> SubscriptionTokenSeaOrmRepository {
        let url_without_db = "postgres://subscription:welcome@localhost:15432";

        if !isolated {
            let url = format!("{}/{}", &url_without_db, "subscription");
            let pool = sea_orm::Database::connect(&url).await.unwrap();

            return SubscriptionTokenSeaOrmRepository::new(pool);
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

        SubscriptionTokenSeaOrmRepository::new(pool)
    }

    #[tokio::test]
    async fn fetching_by_token_after_saving_via_repository_makes_the_same_subscription_token() {
        // given
        let repository = get_repository(false).await;
        let subscriber_id = Uuid::new_v4();
        let subscription_token = SubscriptionToken::issue(subscriber_id);

        // when
        repository.save(&subscription_token).await.unwrap();

        // then
        let saved_subscription_token = repository
            .find_by_token(&subscription_token.token)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(saved_subscription_token.token, subscription_token.token);
        assert_eq!(
            saved_subscription_token.subscriber_id,
            subscription_token.subscriber_id
        );
    }

    #[tokio::test]
    async fn saving_duplicate_token_is_not_allowed() {
        // given
        let repository = get_repository(false).await;
        let subscription_token_1 = SubscriptionToken::issue(Uuid::new_v4());
        let mut subscription_token_2 = SubscriptionToken::issue(Uuid::new_v4());
        subscription_token_2.token = subscription_token_1.token.clone();

        repository.save(&subscription_token_1).await.unwrap();

        // when
        let error = repository.save(&subscription_token_2).await.unwrap_err();

        // then
        assert!(matches!(
            error,
            SubscriptionTokenError::RepositoryOperationFailed(..)
        ));
        assert!(error
            .source()
            .unwrap()
            .to_string()
            .contains("duplicate key value violates unique constraint"));
    }

    #[tokio::test]
    async fn fetching_by_subscriber_id_after_saving_returns_entity() {
        // given
        let repository = get_repository(false).await;
        let subscriber_id = Uuid::new_v4();
        let subscription_token = SubscriptionToken::issue(subscriber_id);

        repository.save(&subscription_token).await.unwrap();

        // when
        let persisted_subscription_token = repository
            .find_by_subscriber_id(subscriber_id)
            .await
            .unwrap()
            .unwrap();

        // then
        assert_eq!(subscription_token.token, persisted_subscription_token.token);
        assert_eq!(
            subscription_token.subscriber_id,
            persisted_subscription_token.subscriber_id
        );
    }
}
