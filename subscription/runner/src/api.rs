use std::sync::Arc;
use std::time::Duration;

use domain::prelude::SubscriberCommandExecutor;
use secrecy::ExposeSecret;

use crate::configuration;

pub async fn run(configuration: configuration::Configuration) {
    // configure application listener
    let listener = configuration::bind_listener(&configuration).await;

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
    let subscription_token_repository =
        repositories::prelude::SubscriptionTokenSeaOrmRepository::new(
            database_connection_pool.clone(),
        );

    // configure email service client for messenger
    let mut headers = messengers::prelude::http_header::HeaderMap::new();
    headers.insert(
        messengers::prelude::http_header::AUTHORIZATION,
        messengers::prelude::http_header::HeaderValue::from_str(
            configuration.messenger.email.api_key.expose_secret(),
        )
        .expect("Failed to parse email server's API key"),
    );

    let subscriber_messenger = messengers::prelude::SubscriberEmailMessenger::new(
        messengers::prelude::HttpClient::builder()
            .default_headers(headers)
            .timeout(Duration::from_secs(
                configuration.messenger.pool_options.connection_timeout,
            ))
            .connect_timeout(Duration::from_secs(
                configuration.messenger.pool_options.request_timeout,
            ))
            .build()
            .expect("Failed to create email client pool"),
        messengers::prelude::HttpUrl::parse(configuration.messenger.email.url.as_ref())
            .expect("Failed to parse email server's URL"),
        configuration.messenger.email.sender,
    );

    // configure container which of the application context
    let container = api::runner::Container {
        subscriber_repository: Arc::new(subscriber_repository.clone()),
        subscription_token_repository: Arc::new(subscription_token_repository),
        subscriber_command_executor: SubscriberCommandExecutor::new(
            subscriber_repository.clone(),
            subscriber_messenger.clone(),
            configuration.application.exposing_address.url,
        ),
        subscriber_messenger: Arc::new(subscriber_messenger),
    };

    // run the application api
    api::runner::run(listener, container).await;
}
