use std::sync::Arc;

use tokio::net::TcpListener;

use crate::{api::router, domain::subscriber::repository::SubscriberRepository};

#[derive(Clone)]
pub struct Container {
    pub subscriber_repository: Arc<dyn SubscriberRepository>,
}

pub async fn run(listener: TcpListener, container: Container) {
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
