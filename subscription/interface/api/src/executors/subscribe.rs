use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;
use uuid::Uuid;

use domain::prelude::{
    SubscriberCommand,
    SubscriberCommandExecutor,
    SubscriberError,
    SubscriberMessenger,
    SubscriberRepository,
    SubscriptionToken,
    SubscriptionTokenCommand,
    SubscriptionTokenCommandExecutor,
    SubscriptionTokenRepository,
};

use crate::error::ApiError;

#[readonly::make]
#[derive(serde::Deserialize, Debug)]
pub struct Request {
    email: String,
    name: String,
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(subscriber_command_executor, subscription_token_command_executor)
)]
pub async fn execute(
    State(subscriber_command_executor): State<
        SubscriberCommandExecutor<impl SubscriberRepository, impl SubscriberMessenger>,
    >,
    State(subscription_token_command_executor): State<
        SubscriptionTokenCommandExecutor<impl SubscriptionTokenRepository>,
    >,
    Form(request): Form<Request>,
) -> Result<StatusCode, ApiError> {
    let subscriber_id = Uuid::new_v4();
    let register_subscriber_command = SubscriberCommand::RegisterSubscriber {
        id: subscriber_id,
        email: request.email,
        name: request.name,
    };
    subscriber_command_executor
        .execute(register_subscriber_command)
        .await
        .map_err(|error| match error {
            SubscriberError::InvalidSubscriberName
            | SubscriberError::InvalidSubscriberEmail
            | SubscriberError::InvalidSubscriberStatus
            | SubscriberError::SubscriberNotFound(_) => {
                ApiError::new(StatusCode::BAD_REQUEST, error.into())
            }
            SubscriberError::RepositoryOperationFailed(_)
            | SubscriberError::MessengerOperationFailed(_)
            | SubscriberError::Unexpected(_) => {
                ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error.into())
            }
        })?;

    let token = SubscriptionToken::generate_token().await;
    let issue_subscription_token_command = SubscriptionTokenCommand::IssueSubscriptionToken {
        token: token.clone(),
        subscriber_id,
    };
    subscription_token_command_executor
        .execute(issue_subscription_token_command)
        .await
        .context("Failed to issue a subscription token")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    let send_confirmation_message_command = SubscriberCommand::SendConfirmationMessage {
        id: subscriber_id,
        token,
    };
    subscriber_command_executor
        .execute(send_confirmation_message_command)
        .await
        .context("Failed to send a confirmation email")
        .map_err(|error| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, error))?;

    Ok(StatusCode::CREATED)
}

#[cfg(test)]
mod tests {
    use domain::prelude::{
        MockSubscriberMessenger,
        MockSubscriberRepository,
        MockSubscriptionTokenRepository,
        Subscriber,
        SubscriberEmail,
        SubscriberName,
    };
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::name::en::FirstName;
    use fake::Fake;

    use super::*;

    #[tokio::test]
    async fn subscription_with_invalid_email_returns_bad_request() {
        // given
        let subscriber_repository = MockSubscriberRepository::new();
        let subscriber_messenger = MockSubscriberMessenger::new();
        let exposing_address = "http://localhost:3000".to_string();
        let subscription_token_repository = MockSubscriptionTokenRepository::new();

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );
        let subscription_token_command_executor =
            SubscriptionTokenCommandExecutor::new(subscription_token_repository);

        // when
        let request = Request {
            email: "not-an-email".to_string(),
            name: FirstName().fake(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(subscription_token_command_executor),
            Form(request),
        )
        .await;

        // then
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn subscription_with_duplicate_email_returns_bad_request() {
        // given
        let mut subscriber_repository = MockSubscriberRepository::new();
        let subscriber_messenger = MockSubscriberMessenger::new();
        let subscription_token_repository = MockSubscriptionTokenRepository::new();
        let exposing_address = "http://localhost:3000".to_string();

        subscriber_repository
            .expect_save()
            .once()
            .returning(|_| Err(SubscriberError::InvalidSubscriberEmail));

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );
        let subscription_token_command_executor =
            SubscriptionTokenCommandExecutor::new(subscription_token_repository);

        // when
        let request = Request {
            email: SafeEmail().fake(),
            name: FirstName().fake(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(subscription_token_command_executor),
            Form(request),
        )
        .await;

        // then
        assert!(response.is_err());
        assert_eq!(response.unwrap_err().code, StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn subscription_returns_200_when_infrastructure_succeed() {
        // given
        let mut subscriber_repository = MockSubscriberRepository::new();
        let mut subscriber_messenger = MockSubscriberMessenger::new();
        let mut subscription_token_repository = MockSubscriptionTokenRepository::new();
        let exposing_address = "http://localhost:3000".to_string();

        subscriber_repository
            .expect_save()
            .once()
            .returning(|_| Ok(()));
        subscriber_repository
            .expect_find_by_id()
            .once()
            .returning(|id| {
                Ok(Some(Subscriber::new(
                    id,
                    SubscriberEmail::parse(SafeEmail().fake()).unwrap(),
                    SubscriberName::parse(FirstName().fake()).unwrap(),
                )))
            });
        subscriber_messenger
            .expect_send()
            .once()
            .returning(|_, _, _| Ok(()));
        subscription_token_repository
            .expect_save()
            .once()
            .returning(|_| Ok(()));

        let subscriber_command_executor = SubscriberCommandExecutor::new(
            subscriber_repository,
            subscriber_messenger,
            exposing_address,
        );
        let subscription_token_command_executor =
            SubscriptionTokenCommandExecutor::new(subscription_token_repository);

        // when
        let request = Request {
            email: SafeEmail().fake(),
            name: FirstName().fake(),
        };
        let response = execute(
            State(subscriber_command_executor),
            State(subscription_token_command_executor),
            Form(request),
        )
        .await;

        // then
        assert!(response.is_ok());
        assert_eq!(response.unwrap(), StatusCode::CREATED);
    }
}
