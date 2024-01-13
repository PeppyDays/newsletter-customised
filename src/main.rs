use std::sync::Arc;
use std::time::Duration;

use secrecy::ExposeSecret;

use newsletter::api;
use newsletter::configuration;
use newsletter::infrastructure::messengers;
use newsletter::infrastructure::repositories;
use newsletter::telemetry;

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
    let subscriber_repository = repositories::SubscriberPostgresRepository::new(
        sqlx::postgres::PgPoolOptions::new()
            .min_connections(configuration.database.pool_options.min_connections)
            .max_connections(configuration.database.pool_options.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(
                configuration.database.pool_options.acquire_timeout,
            ))
            .connect(&configuration.database.connection_string_without_database())
            .await
            .expect("Failed to create repository connection pool"),
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

    let subscriber_messenger = messengers::SubscriberEmailMessenger::new(
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
        reqwest::Url::parse(configuration.messenger.email.host.as_ref())
            .expect("Failed to parse email server's URL"),
        configuration.messenger.email.sender,
    );

    // configure container which of the application context
    let container = api::runner::Container {
        subscriber_repository: Arc::new(subscriber_repository),
        subscriber_messenger: Arc::new(subscriber_messenger),
    };

    api::runner::run(listener, container).await
}
