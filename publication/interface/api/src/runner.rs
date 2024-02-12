use tokio::net::TcpListener;

use domain::prelude::SubscriberRepository;

use crate::container::Container;
use crate::router;

pub async fn run<R>(listener: TcpListener, container: Container<R>)
where
    R: SubscriberRepository + Clone + Send + Sync + 'static,
{
    let app = router::get_router(container).await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
