use tokio::net::TcpListener;

pub struct Configuration {
    pub application: ApplicationConfiguration,
}

pub struct ApplicationConfiguration {
    pub host: String,
    pub port: u16,
}

pub async fn get_configuration() -> Configuration {
    // TODO: wire up the actual configuration later
    Configuration {
        application: ApplicationConfiguration {
            host: String::from("127.0.0.1"),
            port: 8080,
        },
    }
}

pub async fn get_listener(configuration: &Configuration) -> TcpListener {
    TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port,
    ))
    .await
    .expect("Failed to bind a port for application")
}
