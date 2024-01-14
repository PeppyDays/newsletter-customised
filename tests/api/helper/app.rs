use std::net::{
    Ipv4Addr,
    SocketAddr,
    SocketAddrV4,
};
use std::sync::Arc;
use std::time::Duration;

use fake::Fake;
use newsletter::infrastructure::messengers::SubscriberEmailMessenger;
use newsletter::infrastructure::repositories::{
    SubscriberPostgresRepository,
    SubscriptionTokenPostgresRepository,
};
use newsletter::{
    api,
    configuration,
    telemetry,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::{
    Connection,
    Executor,
    PgConnection,
};
use tokio::net::TcpListener;
use wiremock::MockServer;

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
    pub subscriber_repository: Arc<SubscriberPostgresRepository>,
    // subscription token repository for checking data in the database
    pub subscription_token_repository: Arc<SubscriptionTokenPostgresRepository>,
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
        let database_postfix = 10.fake::<String>();
        configuration.database.source.database = format!("{}_{}", "test", database_postfix);

        // configure database
        let mut connection =
            PgConnection::connect(&configuration.database.connection_string_without_database())
                .await
                .unwrap();

        connection
            .execute(
                format!(
                    r#"CREATE DATABASE "{}";"#,
                    configuration.database.source.database
                )
                .as_str(),
            )
            .await
            .unwrap();

        // create a connection pool for migration and repositories
        let pool = sqlx::Pool::<sqlx::Postgres>::connect(
            &configuration.database.connection_string_with_database(),
        )
        .await
        .unwrap();

        // migrate schema changes
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();

        // create repository
        let subscriber_repository = SubscriberPostgresRepository::new(pool.clone());
        let subscription_token_repository = SubscriptionTokenPostgresRepository::new(pool.clone());

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
    // TODO: Modify parameters to the form
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
    // TODO: Modify parameters argument to the token
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
