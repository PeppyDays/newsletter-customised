use chrono::{
    DateTime,
    Utc,
};
use sqlx::postgres::PgRow;
use sqlx::{
    Executor,
    Pool,
    Postgres,
    Row,
};
use uuid::Uuid;

use crate::domain::subscription::subscription_token::error::SubscriptionTokenError;
use crate::domain::subscription::subscription_token::model::SubscriptionToken;
use crate::domain::subscription::subscription_token::repository::SubscriptionTokenRepository;

struct SubscriptionTokenDataModel {
    token: String,
    subscriber_id: Uuid,
    issued_at: DateTime<Utc>,
    expired_at: DateTime<Utc>,
}

impl SubscriptionTokenDataModel {
    pub fn new(
        token: String,
        subscriber_id: Uuid,
        issued_at: DateTime<Utc>,
        expired_at: DateTime<Utc>,
    ) -> Self {
        Self {
            token,
            subscriber_id,
            issued_at,
            expired_at,
        }
    }
}

impl From<&SubscriptionToken> for SubscriptionTokenDataModel {
    fn from(subscription_token: &SubscriptionToken) -> Self {
        Self::new(
            subscription_token.token.clone(),
            subscription_token.subscriber_id,
            subscription_token.issued_at,
            subscription_token.expired_at,
        )
    }
}

impl From<&PgRow> for SubscriptionTokenDataModel {
    fn from(row: &PgRow) -> Self {
        Self {
            token: row.get(0),
            subscriber_id: row.get(1),
            issued_at: row.get(2),
            expired_at: row.get(3),
        }
    }
}

impl TryFrom<SubscriptionTokenDataModel> for SubscriptionToken {
    type Error = SubscriptionTokenError;

    fn try_from(value: SubscriptionTokenDataModel) -> Result<Self, Self::Error> {
        Ok(Self {
            token: value.token,
            subscriber_id: value.subscriber_id,
            issued_at: value.issued_at,
            expired_at: value.expired_at,
        })
    }
}

#[derive(Clone)]
pub struct SubscriptionTokenPostgresRepository {
    pool: Pool<Postgres>,
}

impl SubscriptionTokenPostgresRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SubscriptionTokenRepository for SubscriptionTokenPostgresRepository {
    #[tracing::instrument(name = "Saving subscriber token details", skip(self))]
    async fn save(
        &self,
        subscription_token: &SubscriptionToken,
    ) -> Result<(), SubscriptionTokenError> {
        let data_model = SubscriptionTokenDataModel::from(subscription_token);
        let query = sqlx::query!(
            "INSERT INTO subscription_tokens (token, subscriber_id, issued_at, expired_at) VALUES ($1, $2, $3, $4)",
            data_model.token,
            data_model.subscriber_id,
            data_model.issued_at,
            data_model.expired_at,
        );

        self.pool
            .acquire()
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?
            .execute(query)
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?;

        Ok(())
    }

    #[tracing::instrument(name = "Searching subscriber token details by token", skip(self))]
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SubscriptionToken>, SubscriptionTokenError> {
        let query = sqlx::query!(
            "SELECT token, subscriber_id, issued_at, expired_at FROM subscription_tokens WHERE token = $1",
            token
        );

        let optional_data_model = self
            .pool
            .acquire()
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?
            .fetch_optional(query)
            .await
            .map_err(|error| SubscriptionTokenError::RepositoryOperationFailed(error.into()))?
            .map(|row| SubscriptionTokenDataModel::from(&row));

        match optional_data_model {
            Some(data_model) => Ok(Some(SubscriptionToken::try_from(data_model)?)),
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use uuid::Uuid;

    use crate::configuration::get_configuration;
    use crate::domain::subscription::subscription_token::error::SubscriptionTokenError;
    use crate::domain::subscription::subscription_token::model::SubscriptionToken;
    use crate::domain::subscription::subscription_token::repository::SubscriptionTokenRepository;
    use crate::infrastructure::repositories::SubscriptionTokenPostgresRepository;

    async fn get_repository() -> SubscriptionTokenPostgresRepository {
        let configuration = get_configuration().await;

        SubscriptionTokenPostgresRepository::new(
            sqlx::Pool::connect(&configuration.database.connection_string_with_database())
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
