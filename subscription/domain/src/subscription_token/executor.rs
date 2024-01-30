use uuid::Uuid;

use crate::subscription_token::error::SubscriptionTokenError;
use crate::subscription_token::model::SubscriptionToken;
use crate::subscription_token::repository::SubscriptionTokenRepository;

pub enum SubscriptionTokenCommand {
    IssueSubscriptionToken { token: String, subscriber_id: Uuid },
}

#[derive(Clone)]
pub struct SubscriptionTokenCommandExecutor<R>
where
    R: SubscriptionTokenRepository,
{
    repository: R,
}

impl<R> SubscriptionTokenCommandExecutor<R>
where
    R: SubscriptionTokenRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: SubscriptionTokenCommand,
    ) -> Result<(), SubscriptionTokenError> {
        match command {
            SubscriptionTokenCommand::IssueSubscriptionToken {
                token,
                subscriber_id,
            } => {
                let subscription_token = SubscriptionToken::new(token, subscriber_id);
                self.repository.save(&subscription_token).await
            }
        }
    }
}
