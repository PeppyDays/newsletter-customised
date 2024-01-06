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
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize)]
pub struct DatabaseConfiguration {
    pub engine: DatabaseEngine,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Secret<String>,
    pub database: String,
}

#[derive(serde::Deserialize)]
pub enum DatabaseEngine {
    MySQL,
    PostgreSQL,
}

impl DatabaseConfiguration {
    pub fn connection_string_without_database(&self) -> String {
        let engine = match self.engine {
            DatabaseEngine::MySQL => "mysql",
            DatabaseEngine::PostgreSQL => "postgresql",
        };

        format!(
            "{}://{}:{}@{}:{}",
            engine,
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
        )
    }
    pub fn connection_string_with_database(&self) -> String {
        format!(
            "{}/{}",
            self.connection_string_without_database(),
            self.database,
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
        configuration.application.host, configuration.application.port,
    ))
    .await
    .expect("Failed to bind a port for application")
}
