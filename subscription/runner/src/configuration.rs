use std::collections::HashMap;

use confique::Config;
use secrecy::{
    ExposeSecret,
    Secret,
};
use tokio::net::TcpListener;

#[derive(Debug, Config, Clone)]
pub struct Configuration {
    #[config(nested)]
    pub application: ApplicationConfiguration,

    #[config(nested)]
    pub database: DatabaseConfiguration,

    #[config(nested)]
    pub messenger: MessengerConfiguration,

    #[config(nested)]
    pub logging: LoggingConfiguration,
}

#[derive(Debug, Config, Clone)]
pub struct ApplicationConfiguration {
    #[config(nested)]
    pub listening_address: ApplicationListeningAddress,

    #[config(nested)]
    pub exposing_address: ApplicationExposingAddress,
}

#[derive(Debug, Config, Clone)]
pub struct ApplicationListeningAddress {
    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_HOST")]
    pub host: String,

    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_PORT")]
    pub port: u16,
}

#[derive(Debug, Config, Clone)]
pub struct ApplicationExposingAddress {
    #[config(env = "APP_APPLICATION_EXPOSING_ADDRESS_URL")]
    pub url: String,
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
pub struct MessengerConfiguration {
    #[config(nested)]
    pub email: EmailService,

    #[config(nested)]
    pub pool_options: EmailClientPoolOptions,
}

#[derive(Debug, Config, Clone)]
pub struct EmailService {
    pub url: String,
    pub api_key: Secret<String>,
    pub sender: String,
}

#[derive(Debug, Config, Clone)]
pub struct EmailClientPoolOptions {
    pub connection_timeout: u64,
    pub request_timeout: u64,
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
