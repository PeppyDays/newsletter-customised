use config::{Config, Environment, File};
use secrecy::{ExposeSecret, Secret};
use tokio::net::TcpListener;

#[derive(serde::Deserialize)]
pub struct Configuration {
    pub application: ApplicationConfiguration,
    pub database: DatabaseConfiguration,
    pub logging: LoggingConfiguration,
}

#[derive(serde::Deserialize)]
pub struct ApplicationConfiguration {
    pub listening_address: ApplicationListeningAddress,
}

#[derive(serde::Deserialize)]
pub struct ApplicationListeningAddress {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfiguration {
    pub source: DatabaseSource,
    pub pool_options: DatabasePoolOptions,
}

#[derive(serde::Deserialize)]
pub enum DatabaseEngine {
    MySQL,
    PostgreSQL,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSource {
    pub engine: DatabaseEngine,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database: String,
}

#[derive(serde::Deserialize)]
pub struct DatabasePoolOptions {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: u64,
}

impl DatabaseConfiguration {
    pub fn connection_string_without_database(&self) -> String {
        let engine = match self.source.engine {
            DatabaseEngine::MySQL => "mysql",
            DatabaseEngine::PostgreSQL => "postgresql",
        };

        format!(
            "{}://{}:{}@{}:{}",
            engine,
            self.source.username,
            self.source.password.expose_secret(),
            self.source.host,
            self.source.port,
        )
    }
    pub fn connection_string_with_database(&self) -> String {
        format!(
            "{}/{}",
            self.connection_string_without_database(),
            self.source.database,
        )
    }
}

#[derive(serde::Deserialize)]
pub struct LoggingConfiguration {
    pub level: String,
}

pub async fn get_configuration() -> Configuration {
    let config = Config::builder()
        .add_source(File::with_name("configuration.yaml"))
        .add_source(
            Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("_"),
        )
        .build()
        .expect("Failed to build configuration");

    config
        .try_deserialize()
        .expect("Failed to deserialize configuration")
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
