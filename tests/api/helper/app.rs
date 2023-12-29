use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use reqwest::Client;
use tokio::net::TcpListener;

use newsletter::api;

pub struct App {
    pub address: SocketAddr,
    pub client: Client,
}

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
