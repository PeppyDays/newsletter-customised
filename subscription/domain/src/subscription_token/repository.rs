use uuid::Uuid;

use crate::subscription_token::error::SubscriptionTokenError;
use crate::subscription_token::model::SubscriptionToken;

#[mockall::automock]
#[async_trait::async_trait]
pub trait SubscriptionTokenRepository: Send + Sync {
    async fn save(
        &self,
        subscription_token: &SubscriptionToken,
    ) -> Result<(), SubscriptionTokenError>;
    async fn find_by_token(
        &self,
        token: &str,
    ) -> Result<Option<SubscriptionToken>, SubscriptionTokenError>;
    async fn find_by_subscriber_id(
        &self,
        subscriber_id: Uuid,
    ) -> Result<Option<SubscriptionToken>, SubscriptionTokenError>;
}
