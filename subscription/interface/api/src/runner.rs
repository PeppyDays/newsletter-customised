use std::sync::Arc;

use axum::extract::FromRef;
use domain::prelude::{
    SubscriberCommandExecutor,
    SubscriberMessenger,
    SubscriberRepository,
    SubscriptionTokenRepository,
};
use tokio::net::TcpListener;

use crate::router;

// TODO: Add a new method
#[derive(Clone)]
pub struct Container<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
    pub subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
    pub subscriber_command_executor: SubscriberCommandExecutor<R>,
    pub subscriber_messenger: Arc<dyn SubscriberMessenger>,
    pub exposing_address: Arc<String>,
}

impl<R> FromRef<Container<R>> for Arc<dyn SubscriberRepository>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscriber_repository.clone()
    }
}

impl<R> FromRef<Container<R>> for Arc<dyn SubscriberMessenger>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscriber_messenger.clone()
    }
}

impl<R> FromRef<Container<R>> for SubscriberCommandExecutor<R>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscriber_command_executor.clone()
    }
}

impl<R> FromRef<Container<R>> for Arc<dyn SubscriptionTokenRepository>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.subscription_token_repository.clone()
    }
}

impl<R> FromRef<Container<R>> for Arc<String>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R>) -> Self {
        container.exposing_address.clone()
    }
}

pub async fn run<R>(listener: TcpListener, container: Container<R>)
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
