use axum::extract::FromRef;
use tokio::net::TcpListener;

use domain::prelude::{
    SubscriberCommandExecutor,
    SubscriberMessenger,
    SubscriberQueryReader,
    SubscriberRepository,
    SubscriptionTokenCommandExecutor,
    SubscriptionTokenQueryReader,
    SubscriptionTokenRepository,
};

use crate::router;

#[derive(Clone)]
pub struct Container<R, M, T>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    subscriber_command_executor: SubscriberCommandExecutor<R, M>,
    subscriber_query_reader: SubscriberQueryReader<R>,
    subscription_token_command_executor: SubscriptionTokenCommandExecutor<T>,
    subscription_token_query_reader: SubscriptionTokenQueryReader<T>,
}

impl<R, M, T> Container<R, M, T>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    pub fn new(
        subscriber_repository: R,
        subscriber_messenger: M,
        subscription_token_repository: T,
        exposing_address: String,
    ) -> Self {
        Self {
            subscriber_command_executor: SubscriberCommandExecutor::new(
                subscriber_repository.clone(),
                subscriber_messenger.clone(),
                exposing_address,
            ),
            subscriber_query_reader: SubscriberQueryReader::new(subscriber_repository.clone()),
            subscription_token_command_executor: SubscriptionTokenCommandExecutor::new(
                subscription_token_repository.clone(),
            ),
            subscription_token_query_reader: SubscriptionTokenQueryReader::new(
                subscription_token_repository.clone(),
            ),
        }
    }
}

impl<R, M, T> FromRef<Container<R, M, T>> for SubscriberQueryReader<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M, T>) -> Self {
        container.subscriber_query_reader.clone()
    }
}

impl<R, M, T> FromRef<Container<R, M, T>> for SubscriberCommandExecutor<R, M>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M, T>) -> Self {
        container.subscriber_command_executor.clone()
    }
}

impl<R, M, T> FromRef<Container<R, M, T>> for SubscriptionTokenCommandExecutor<T>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M, T>) -> Self {
        container.subscription_token_command_executor.clone()
    }
}

impl<R, M, T> FromRef<Container<R, M, T>> for SubscriptionTokenQueryReader<T>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M, T>) -> Self {
        container.subscription_token_query_reader.clone()
    }
}

pub async fn run<R, M, T>(listener: TcpListener, container: Container<R, M, T>)
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
