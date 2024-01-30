use std::sync::Arc;

use axum::extract::FromRef;
use domain::prelude::{
    SubscriberCommandExecutor, SubscriberMessenger, SubscriberQueryReader, SubscriberRepository,
    SubscriptionTokenCommandExecutor, SubscriptionTokenRepository,
};
use tokio::net::TcpListener;

use crate::router;

// TODO: Add a new method
#[derive(Clone)]
pub struct Container<R, M, T>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    T: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    pub subscriber_command_executor: SubscriberCommandExecutor<R, M>,
    pub subscriber_query_reader: SubscriberQueryReader<R>,
    pub subscription_token_command_executor: SubscriptionTokenCommandExecutor<T>,
    pub subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
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
        container.subscription_token_repository.clone()
    }
}

pub async fn run<R, M, T>(listener: TcpListener, container: Container<R, M, T>)
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
    P: SubscriptionTokenRepository + Clone + Send + Sync + 'static,
{
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
