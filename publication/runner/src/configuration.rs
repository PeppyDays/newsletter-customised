use std::collections::HashMap;

use confique::Config;
use secrecy::{
    ExposeSecret,
    Secret,
};

#[derive(Debug, Config, Clone)]
pub struct Configuration {
    #[config(nested)]
    pub api: ApiConfiguration,

    #[config(nested)]
    pub database: DatabaseConfiguration,

    #[config(nested)]
    pub gateways: GatewaysConfiguration,

    #[config(nested)]
    pub logging: LoggingConfiguration,
}

#[derive(Debug, Config, Clone)]
pub struct ApiConfiguration {
    #[config(nested)]
    pub listening: ApiListening,
}

#[derive(Debug, Config, Clone)]
pub struct ApiListening {
    #[config(env = "APP_API_LISTENING_HOST")]
    pub host: String,

    #[config(env = "APP_API_LISTENING_PORT")]
    pub port: u16,
}

#[derive(Debug, Config, Clone)]
pub struct DatabaseConfiguration {
    #[config(nested)]
    pub source: DatabaseSource,

    #[config(nested)]
    pub pool_options: DatabasePoolOptions,
}

#[derive(Debug, Config, Clone)]
pub struct DatabaseSource {
    #[config(env = "APP_DATABASE_SOURCE_HOST")]
    pub host: String,

    #[config(env = "APP_DATABASE_SOURCE_PORT")]
    pub port: u16,

    #[config(env = "APP_DATABASE_SOURCE_USERNAME")]
    pub username: String,

    #[config(env = "APP_DATABASE_SOURCE_PASSWORD")]
    pub password: Secret<String>,

    #[config(env = "APP_DATABASE_SOURCE_DATABASE")]
    pub database: String,
}

#[derive(Debug, Config, Clone)]
pub struct DatabasePoolOptions {
    pub min_connections: u32,
    pub max_connections: u32,
    pub connect_timeout: u64,
}

impl DatabaseConfiguration {
    pub fn connection_string_without_database(&self) -> Secret<String> {
        Secret::new(format!(
            "postgresql://{}:{}@{}:{}",
            self.source.username,
            self.source.password.expose_secret(),
            self.source.host,
            self.source.port,
        ))
    }
    pub fn connection_string_with_database(&self) -> Secret<String> {
        Secret::new(format!(
            "{}/{}",
            self.connection_string_without_database().expose_secret(),
            self.source.database,
        ))
    }
}

#[derive(Debug, Config, Clone)]
pub struct GatewaysConfiguration {
    #[config(nested)]
    pub subscription: SubscriptionGatewayAddress,
}

#[derive(Debug, Config, Clone)]
pub struct SubscriptionGatewayAddress {
    #[config(env = "APP_GATEWAYS_SUBSCRIPTION_ORIGIN")]
    pub origin: String,
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
