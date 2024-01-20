use sea_orm::entity::prelude::*;
use sea_orm::ActiveValue;
use uuid::Uuid;

use crate::domain::subscription::subscription_token::error::SubscriptionTokenError;
use crate::domain::subscription::subscription_token::model::SubscriptionToken;
use crate::domain::subscription::subscription_token::repository::SubscriptionTokenRepository;

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
    #[tracing::instrument(name = "Saving subscriber token details", skip(self))]
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

    #[tracing::instrument(name = "Searching subscriber token details by token", skip(self))]
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
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use sea_orm::Database;
    use uuid::Uuid;

    use crate::configuration::get_configuration;
    use crate::domain::subscription::subscription_token::error::SubscriptionTokenError;
    use crate::domain::subscription::subscription_token::model::SubscriptionToken;
    use crate::domain::subscription::subscription_token::repository::SubscriptionTokenRepository;

    use super::*;

    async fn get_repository() -> SubscriptionTokenSeaOrmRepository {
        let configuration = get_configuration().await;

        SubscriptionTokenSeaOrmRepository::new(
            Database::connect(&configuration.database.connection_string_without_database())
                .await
                .unwrap(),
        )
    }

    #[tokio::test]
    async fn fetching_by_token_after_saving_via_repository_makes_the_same_subscription_token() {
        // given
        let repository = get_repository().await;
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
        let repository = get_repository().await;
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
}
