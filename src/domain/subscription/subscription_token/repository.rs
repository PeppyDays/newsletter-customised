use crate::domain::subscription::subscription_token::error::SubscriptionTokenError;
use crate::domain::subscription::subscription_token::model::SubscriptionToken;

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
}
