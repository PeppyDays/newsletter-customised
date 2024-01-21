use std::sync::Arc;

use tokio::net::TcpListener;

use crate::api::router;
use crate::configuration::ApplicationExposingAddress;
use crate::domain::subscription::subscriber::prelude::{
    SubscriberMessenger,
    SubscriberRepository,
};
use crate::domain::subscription::subscription_token::prelude::SubscriptionTokenRepository;

#[derive(Clone)]
pub struct Container {
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
    pub subscription_token_repository: Arc<dyn SubscriptionTokenRepository>,
    pub subscriber_messenger: Arc<dyn SubscriberMessenger>,
    pub exposing_address: Arc<ApplicationExposingAddress>,
}

pub async fn run(listener: TcpListener, container: Container) {
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
