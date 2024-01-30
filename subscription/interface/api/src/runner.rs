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
pub struct Container<R, M>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
    pub subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
    pub subscriber_command_executor: SubscriberCommandExecutor<R, M>,
    pub subscriber_messenger: Arc<dyn SubscriberMessenger>,
}

impl<R, M> FromRef<Container<R, M>> for Arc<dyn SubscriberRepository>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M>) -> Self {
        container.subscriber_repository.clone()
    }
}

impl<R, M> FromRef<Container<R, M>> for Arc<dyn SubscriberMessenger>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M>) -> Self {
        container.subscriber_messenger.clone()
    }
}

impl<R, M> FromRef<Container<R, M>> for SubscriberCommandExecutor<R, M>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M>) -> Self {
        container.subscriber_command_executor.clone()
    }
}

impl<R, M> FromRef<Container<R, M>> for Arc<dyn SubscriptionTokenRepository>
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    fn from_ref(container: &Container<R, M>) -> Self {
        container.subscription_token_repository.clone()
    }
}

pub async fn run<R, M>(listener: TcpListener, container: Container<R, M>)
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
    M: SubscriberMessenger + Clone + Send + Sync + 'static,
{
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
