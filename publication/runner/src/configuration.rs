use std::collections::HashMap;

use confique::Config;
use tokio::net::TcpListener;

#[derive(Debug, Config, Clone)]
pub struct Configuration {
    #[config(nested)]
    pub application: ApplicationConfiguration,

    #[config(nested)]
    pub gateways: GatewaysConfiguration,

    #[config(nested)]
    pub logging: LoggingConfiguration,
}

#[derive(Debug, Config, Clone)]
pub struct ApplicationConfiguration {
    #[config(nested)]
    pub listening_address: ApplicationListeningAddress,
}

#[derive(Debug, Config, Clone)]
pub struct ApplicationListeningAddress {
    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_HOST")]
    pub host: String,

    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_PORT")]
    pub port: u16,
}

#[derive(Debug, Config, Clone)]
pub struct GatewaysConfiguration {
    #[config(nested)]
    pub subscription: SubscriptionGatewayAddress,
}

#[derive(Debug, Config, Clone)]
pub struct SubscriptionGatewayAddress {
    #[config(env = "APP_GATEWAYS_SUBSCRIPTION_URL")]
    pub url: String,
}

#[derive(Debug, Config, Clone)]
pub struct LoggingConfiguration {
    #[config(env = "APP_LOGGING_GLOBAL")]
    pub global: String,
    pub crates: Option<HashMap<String, String>>,
}

pub async fn get_configuration(file: &str) -> Configuration {
    Configuration::builder()
        .env()
        .file(file)
        .load()
        .expect("Failed to load configuration")
}

pub async fn bind_listener(configuration: &Configuration) -> TcpListener {
    TcpListener::bind(format!(
        "{}:{}",
        configuration.application.listening_address.host,
        configuration.application.listening_address.port,
    ))
    .await
    .expect("Failed to bind a port for application")
}
