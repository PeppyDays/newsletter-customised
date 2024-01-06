use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use fake::Fake;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::Serialize;
use sqlx::{Connection, Executor, PgConnection};
use tokio::net::TcpListener;

use newsletter::{
    api, configuration, infrastructure::repositories::SubscriberPostgresRepository, telemetry,
};

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
    pub client: Client,
    // subscriber repository for checking data in the database
    pub subscriber_repository: Arc<SubscriberPostgresRepository>,
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

        // randomise database for data isolation
        let database_postfix = 10.fake::<String>();
        configuration.database.database = format!("{}_{}", "test", database_postfix);

        // configure database
        let mut connection =
            PgConnection::connect(&configuration.database.connection_string_without_database())
                .await
                .unwrap();

        connection
            .execute(format!(r#"CREATE DATABASE "{}";"#, configuration.database.database).as_str())
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

        // create container for application context
        let subscriber_repository = SubscriberPostgresRepository::new(pool);

        let container = api::runner::Container {
            subscriber_repository: Arc::new(subscriber_repository.clone()),
        };

        // create http client
        let client = Client::new();

        // start a server
        tokio::spawn(api::runner::run(listener, container));

        App {
            address,
            client,
            subscriber_repository: Arc::new(subscriber_repository),
        }
    }
}

// simplify application call testing
impl App {
    // POST /subscribe
    pub async fn post_subscribe<T: Serialize + ?Sized>(&self, parameters: &T) -> reqwest::Response {
        let url = format!("http://{}/subscribe", self.address);
        self.client
            .post(url)
            .form(&parameters)
            .send()
            .await
            .unwrap()
    }
}
