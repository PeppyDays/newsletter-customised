use crate::subscription_token::error::SubscriptionTokenError;
use crate::subscription_token::model::SubscriptionToken;
use crate::subscription_token::repository::SubscriptionTokenRepository;

pub enum SubscriptionTokenQuery {
    InquireSubscriptionTokenByToken { token: String },
}

#[derive(Clone)]
pub struct SubscriptionTokenQueryReader<R>
where
    R: SubscriptionTokenRepository,
{
    repository: R,
}

impl<R> SubscriptionTokenQueryReader<R>
where
    R: SubscriptionTokenRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn read(
        &self,
        query: SubscriptionTokenQuery,
    ) -> Result<SubscriptionToken, SubscriptionTokenError> {
        match query {
            SubscriptionTokenQuery::InquireSubscriptionTokenByToken { token } => self
                .repository
                .find_by_token(&token)
                .await?
                .ok_or(SubscriptionTokenError::SubscriptionTokenNotFound(token)),
        }
    }
}
