use std::sync::Arc;

use newsletter::api;
use newsletter::configuration;
use newsletter::infrastructure::repositories;
use newsletter::telemetry;

#[tokio::main]
async fn main() {
    let configuration = configuration::get_configuration().await;

    // configure logging
    let subscriber =
        telemetry::get_subscriber("newsletter", &configuration.logging.level, std::io::stdout);
    telemetry::initialize_subscriber(subscriber);

    // configure application listener
    let listener = configuration::bind_listener(&configuration).await;

    // configure database connection pool
    let subscriber_repository = repositories::SubscriberPostgresRepository::new(
        sqlx::postgres::PgPoolOptions::new()
            .min_connections(configuration.database.pool_options.min_connections)
            .max_connections(configuration.database.pool_options.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                configuration.database.pool_options.acquire_timeout,
            ))
            .connect(&configuration.database.connection_string_without_database())
            .await
            .unwrap(),
    );

    // configure container which of the application context
    let container = api::runner::Container {
        subscriber_repository: Arc::new(subscriber_repository),
    };

    api::runner::run(listener, container).await
}
