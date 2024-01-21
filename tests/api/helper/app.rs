use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
use std::sync::Arc;
use std::time::Duration;

use fake::Fake;
use once_cell::sync::Lazy;
use sea_orm::ConnectionTrait;
use serde::Serialize;
use tokio::net::TcpListener;
use wiremock::MockServer;

use newsletter::infrastructure::subscription::subscriber::prelude::*;
use newsletter::infrastructure::subscription::subscription_token::prelude::*;
use newsletter::{api, configuration, telemetry};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    let subscriber_name = "newsletter-test";

    // if TEST_LOG environment variable is set, then log to stdout
    // otherwise, log to sink, which is a blackhole
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::initialize_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::initialize_subscriber(subscriber);
    };
});

pub struct App {
    // application address
    pub address: SocketAddr,
    // reqwest client for checking API calls from external client
    pub client: reqwest::Client,
    // mock server for checking email calls from application
    pub email_server: Arc<MockServer>,
    // subscriber repository for checking data in the database
    pub subscriber_repository: Arc<SubscriberSeaOrmRepository>,
    // subscription token repository for checking data in the database
    pub subscription_token_repository: Arc<SubscriptionTokenSeaOrmRepository>,
}

// create a test application
impl App {
    pub async fn new() -> App {
        // configure logging
        Lazy::force(&TRACING);

        // configure listener with randomised port
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("Failed to start an test application");
        let address = listener.local_addr().unwrap();

        // get and manipulate configuration
        let mut configuration = configuration::get_configuration().await;

        // start an email server
        let email_server = MockServer::start().await;
        configuration.messenger.email.url = email_server.uri();

        // randomise database for data isolation
        let database = format!("{}_{}", "test", 10.fake::<String>());
        configuration.database.source.database = database.clone();

        let connection = sea_orm::Database::connect(
            &configuration.database.connection_string_without_database(),
        )
        .await
        .unwrap();

        match connection.get_database_backend() {
            sea_orm::DatabaseBackend::MySql => {
                connection
                    .execute(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::MySql,
                        format!("CREATE SCHEMA IF NOT EXISTS `{}`;", &database),
                    ))
                    .await
                    .unwrap();
            }
            sea_orm::DatabaseBackend::Postgres => {
                connection
                    .execute(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!("DROP DATABASE IF EXISTS \"{}\";", &database),
                    ))
                    .await
                    .unwrap();

                connection
                    .execute(sea_orm::Statement::from_string(
                        sea_orm::DatabaseBackend::Postgres,
                        format!("CREATE DATABASE \"{}\";", &database),
                    ))
                    .await
                    .unwrap();
            }
            sea_orm::DatabaseBackend::Sqlite => (),
        };

        // migrate schema changes
        let pool =
            sea_orm::Database::connect(&configuration.database.connection_string_with_database())
                .await
                .unwrap();

        sqlx::migrate!("./migrations")
            .run(pool.get_postgres_connection_pool())
            .await
            .unwrap();

        // create repository
        let subscriber_repository = SubscriberSeaOrmRepository::new(pool.clone());
        let subscription_token_repository = SubscriptionTokenSeaOrmRepository::new(pool.clone());

        // create email messenger
        let subscriber_messenger = SubscriberEmailMessenger::new(
            reqwest::Client::builder()
                .timeout(Duration::from_secs(
                    configuration.messenger.pool_options.connection_timeout,
                ))
                .connect_timeout(Duration::from_secs(
                    configuration.messenger.pool_options.request_timeout,
                ))
                .build()
                .expect("Failed to create email client pool"),
            reqwest::Url::parse(configuration.messenger.email.url.as_ref()).unwrap(),
            configuration.messenger.email.sender,
        );

        // create container for application context
        let container = api::runner::Container {
            subscriber_repository: Arc::new(subscriber_repository.clone()),
            subscription_token_repository: Arc::new(subscription_token_repository.clone()),
            subscriber_messenger: Arc::new(subscriber_messenger.clone()),
            exposing_address: Arc::new(configuration.application.exposing_address),
        };

        // create http client for accessing application APIs
        let client = reqwest::Client::new();

        tokio::spawn(api::runner::run(listener, container));

        App {
            address,
            client,
            email_server: Arc::new(email_server),
            subscriber_repository: Arc::new(subscriber_repository),
            subscription_token_repository: Arc::new(subscription_token_repository),
        }
    }
}

// simplify application call testing
impl App {
    // POST /subscription/subscribe
    pub async fn post_subscribe<T: Serialize + ?Sized>(&self, parameters: &T) -> reqwest::Response {
        let url = format!("http://{}/subscription/subscribe", self.address);
        self.client
            .post(url)
            .form(&parameters)
            .send()
            .await
            .unwrap()
    }

    // GET /subscription/confirm
    pub async fn get_subscription_confirm<T: Serialize + ?Sized>(
        &self,
        parameters: &T,
    ) -> reqwest::Response {
        let url = format!("http://{}/subscription/confirm", self.address);
        self.client
            .get(url)
            .query(&parameters)
            .send()
            .await
            .unwrap()
    }
}
