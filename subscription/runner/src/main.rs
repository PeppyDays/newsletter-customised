#[tokio::main]
async fn main() {
    // TODO: Implement global logging and apply API's logging level from API's configuration
    let subscriber = runner::telemetry::get_subscriber("subscription", "info", std::io::stdout);
    runner::telemetry::initialize_subscriber(subscriber);

    // run api
    runner::api::run().await;
}
