use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use reqwest::Client;
use serde::Serialize;
use tokio::net::TcpListener;

use newsletter::api;

pub struct App {
    pub address: SocketAddr,
    pub client: Client,
}

// create a test application
impl App {
    pub async fn new() -> App {
        // configure listener with randomised port
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .await
            .expect("Failed to start an test application");
        let address = listener.local_addr().unwrap();

        // create http client
        let client = Client::new();

        // start a server
        tokio::spawn(api::runner::run(listener));

        App { address, client }
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
