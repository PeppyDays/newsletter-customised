use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{fmt::MakeWriter, layer::SubscriberExt, EnvFilter, Registry};

pub fn get_subscriber<Sink>(
    name: &str,
    env_filter: &str,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    // try_from_default_env() will read the RUST_LOG environment variable
    // and use it to set the log level filter.
    // If it is not set, then use env_filter argument.
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));

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
