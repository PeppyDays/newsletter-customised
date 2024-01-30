use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{
    BunyanFormattingLayer,
    JsonStorageLayer,
};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{
    EnvFilter,
    Registry,
};

use crate::configuration::LoggingConfiguration;

pub fn get_subscriber<Sink>(
    name: &str,
    filter: LoggingConfiguration,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // try_from_default_env() will read the RUST_LOG environment variable
    // and use it to set the log level filter.
    // If it is not set, then use env_filter argument.
    let mut env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter.global));

    for (crate_name, logging_level) in filter.crates {
        env_filter =
            env_filter.add_directive(format!("{}={}", crate_name, logging_level).parse().unwrap());
    }

    // configure log format with the application name and target logging output (e.g. std::io::stdout)
    let formatting_layer = BunyanFormattingLayer::new(name.to_string(), sink);

    // based on the format configured above, stores it as JSON.
    // Also propagate tracing context from parent spans to their children.
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn initialize_subscriber(subscriber: impl Subscriber + Send + Sync) {
    // redirect all log events to the tracing subscriber
    LogTracer::init().expect("Failed to set logger");

    // all dependent crate logs are subscribed by the tracing subscriber
    set_global_default(subscriber).expect("Failed to set subscriber");
}
