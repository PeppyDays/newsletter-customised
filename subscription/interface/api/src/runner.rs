use tokio::net::TcpListener;

use domain::prelude::{
    SubscriberMessenger,
    SubscriberRepository,
    SubscriptionTokenRepository,
};

use crate::{
    container,
    router,
};

pub async fn run<R, M, T>(listener: TcpListener, container: container::Container<R, M, T>)
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
