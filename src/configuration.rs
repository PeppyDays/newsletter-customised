use confique::Config;
use secrecy::{ExposeSecret, Secret};
use tokio::net::TcpListener;

#[derive(Debug, Config)]
pub struct Configuration {
    #[config(nested)]
    pub application: ApplicationConfiguration,
    #[config(nested)]
    pub database: DatabaseConfiguration,
    #[config(nested)]
    pub logging: LoggingConfiguration,
}

#[derive(Debug, Config)]
pub struct ApplicationConfiguration {
    #[config(nested)]
    pub listening_address: ApplicationListeningAddress,
}

#[derive(Debug, Config)]
pub struct ApplicationListeningAddress {
    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_HOST")]
    pub host: String,
    #[config(env = "APP_APPLICATION_LISTENING_ADDRESS_PORT")]
    pub port: u16,
}

#[derive(Debug, Config)]
pub struct DatabaseConfiguration {
    #[config(nested)]
    pub source: DatabaseSource,
    #[config(nested)]
    pub pool_options: DatabasePoolOptions,
}

#[derive(Debug, Config)]
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

#[derive(Debug, Config)]
pub struct DatabasePoolOptions {
    pub min_connections: u32,
    pub max_connections: u32,
    pub acquire_timeout: u64,
}

impl DatabaseConfiguration {
    pub fn connection_string_without_database(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}",
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

#[derive(Debug, Config)]
pub struct LoggingConfiguration {
    #[config(env = "APP_LOGGING_LEVEL")]
    pub level: String,
}

pub async fn get_configuration() -> Configuration {
    Configuration::builder()
        .env()
        .file("configuration.yaml")
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
