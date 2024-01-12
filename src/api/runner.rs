use std::sync::Arc;

use tokio::net::TcpListener;

use crate::api::router;
use crate::domain::subscriber::{messenger::SubscriberMessenger, repository::SubscriberRepository};

#[derive(Clone)]
pub struct Container {
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
    pub subscriber_messenger: Arc<dyn SubscriberMessenger>,
}

pub async fn run(listener: TcpListener, container: Container) {
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
