use std::time::Duration;

use secrecy::ExposeSecret;
use tokio::net::TcpListener;

use crate::configuration;

pub async fn run(configuration: &configuration::Configuration) {
    // configure api listener
    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.api.listening.host, configuration.api.listening.port,
    ))
    .await
    .expect("Failed to bind a port for application");

    // configure database connection pool
    let mut database_connection_options = repositories::prelude::DatabaseConnectionOptions::new(
        configuration
            .database
            .connection_string_with_database()
            .expose_secret(),
    );

    database_connection_options
        .min_connections(configuration.database.pool_options.min_connections)
        .max_connections(configuration.database.pool_options.max_connections)
        .connect_timeout(Duration::from_secs(
            configuration.database.pool_options.connect_timeout,
        ))
        .sqlx_logging(true)
        .sqlx_logging_level(tracing_log::log::LevelFilter::Debug)
        .sqlx_slow_statements_logging_settings(
            tracing_log::log::LevelFilter::Warn,
            Duration::from_secs(1),
        );

    let database_connection_pool =
        repositories::prelude::DatabaseConnection::connect(database_connection_options)
            .await
            .expect("Failed to create repository connection pool");

    let subscriber_repository =
        repositories::prelude::SubscriberSeaOrmRepository::new(database_connection_pool.clone());

    // configure container which of the api context
    let container = api::container::Container::new(subscriber_repository);

    // run the api
    api::runner::run(listener, container).await;
}
