use std::sync::Arc;

use newsletter::api;
use newsletter::configuration;
use newsletter::infrastructure::repositories;

#[tokio::main]
async fn main() {
    let configuration = configuration::get_configuration().await;

    let listener = configuration::bind_listener(&configuration).await;

    let subscriber_repository = repositories::SubscriberPostgresRepository::new(
        sqlx::Pool::connect(&configuration.database.connection_string_with_database())
            .await
            .unwrap(),
    );
    let container = api::runner::Container {
        subscriber_repository: Arc::new(subscriber_repository),
    };

    api::runner::run(listener, container).await
}
