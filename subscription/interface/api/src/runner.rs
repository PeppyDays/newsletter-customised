use std::sync::Arc;

use axum::extract::FromRef;
use domain::prelude::{
    SubscriberMessenger,
    SubscriberRepository,
    SubscriptionTokenRepository,
};
use tokio::net::TcpListener;

use crate::configuration::ApplicationExposingAddress;
use crate::router;

#[derive(Clone)]
pub struct Container {
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
    pub subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
    pub subscriber_messenger: Arc<dyn SubscriberMessenger>,
    pub exposing_address: Arc<ApplicationExposingAddress>,
}

impl FromRef<Container> for Arc<dyn SubscriberRepository> {
    fn from_ref(container: &Container) -> Self {
        container.subscriber_repository.clone()
    }
}

impl FromRef<Container> for Arc<dyn SubscriptionTokenRepository> {
    fn from_ref(container: &Container) -> Self {
        container.subscription_token_repository.clone()
    }
}

impl FromRef<Container> for Arc<dyn SubscriberMessenger> {
    fn from_ref(container: &Container) -> Self {
        container.subscriber_messenger.clone()
    }
}

impl FromRef<Container> for Arc<ApplicationExposingAddress> {
    fn from_ref(container: &Container) -> Self {
        container.exposing_address.clone()
    }
}

pub async fn run(listener: TcpListener, container: Container) {
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
