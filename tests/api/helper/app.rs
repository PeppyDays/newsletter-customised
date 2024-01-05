use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
};

use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpListener;

use newsletter::{api, configuration, infrastructure::repositories::SubscriberPostgresRepository};

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
        // configure listener with randomised port
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("Failed to start an test application");
        let address = listener.local_addr().unwrap();

        // create container for application context
        let configuration = configuration::get_configuration().await;
        let subscriber_repository = SubscriberPostgresRepository::new(
            sqlx::Pool::connect(&configuration.database.connection_string())
                .await
                .unwrap(),
        );
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
