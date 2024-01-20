use std::sync::Arc;
use std::time::Duration;

use newsletter::{api, configuration, infrastructure, telemetry};
use sea_orm::{ConnectOptions, Database};
use secrecy::ExposeSecret;

#[tokio::main]
async fn main() {
    let configuration = configuration::get_configuration().await;

    // configure logging
    let subscriber =
        telemetry::get_subscriber("newsletter", &configuration.logging.level, std::io::stdout);
    telemetry::initialize_subscriber(subscriber);

    // log the configuration
    tracing::debug!("{:?}", configuration);

    // configure application listener
    let listener = configuration::bind_listener(&configuration).await;

    // configure database connection pool
    let database_connection_pool = sqlx::postgres::PgPoolOptions::new()
        .min_connections(configuration.database.pool_options.min_connections)
        .max_connections(configuration.database.pool_options.max_connections)
        .acquire_timeout(Duration::from_secs(
            configuration.database.pool_options.acquire_timeout,
        ))
        .connect(&configuration.database.connection_string_without_database())
        .await
        .expect("Failed to create repository connection pool");

    let mut option = ConnectOptions::new(configuration.database.connection_string_with_database());
    option
        .min_connections(configuration.database.pool_options.min_connections)
        .max_connections(configuration.database.pool_options.max_connections)
        .connect_timeout(Duration::from_secs(
            configuration.database.pool_options.acquire_timeout,
        ))
        .sqlx_logging(true);

    let pool = Database::connect(option)
        .await
        .expect("Failed to create repository connection pool");

    let subscriber_repository =
        infrastructure::subscription::subscriber::SubscriberPostgresRepository::new(
            database_connection_pool.clone(),
        );
    let subscription_token_repository =
        infrastructure::subscription::subscription_token::SubscriptionTokenSeaOrmRepository::new(
            pool.clone(),
        );

    // configure email service client for messenger
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(
            configuration.messenger.email.api_key.expose_secret(),
        )
        .expect("Failed to parse email server's API key"),
    );

    let subscriber_messenger =
        infrastructure::subscription::subscriber::SubscriberEmailMessenger::new(
            reqwest::Client::builder()
                .default_headers(headers)
                .timeout(Duration::from_secs(
                    configuration.messenger.pool_options.connection_timeout,
                ))
                .connect_timeout(Duration::from_secs(
                    configuration.messenger.pool_options.request_timeout,
                ))
                .build()
                .expect("Failed to create email client pool"),
            reqwest::Url::parse(configuration.messenger.email.url.as_ref())
                .expect("Failed to parse email server's URL"),
            configuration.messenger.email.sender,
        );

    // configure container which of the application context
    let container = api::runner::Container {
        subscriber_repository: Arc::new(subscriber_repository),
        subscription_token_repository: Arc::new(subscription_token_repository),
        subscriber_messenger: Arc::new(subscriber_messenger),
        exposing_address: Arc::new(configuration.application.exposing_address),
    };

    api::runner::run(listener, container).await
}
