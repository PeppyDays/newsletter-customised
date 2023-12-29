use tokio::net::TcpListener;

use crate::api::router;

pub async fn run(listener: TcpListener) {
    let app = router::get_router().await;

    axum::serve(listener, app)
        .await
        .expect("Failed to start up the application");
}
