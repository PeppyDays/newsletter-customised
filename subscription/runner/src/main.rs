use runner::configuration::get_configuration;

#[tokio::main]
async fn main() {
    let configuration = get_configuration("configuration.yaml").await;

    let subscriber = runner::telemetry::get_subscriber(
        "subscription",
        configuration.logging.clone(),
        std::io::stdout,
    );
    runner::telemetry::initialize_subscriber(subscriber);

    // run api
    runner::api::run(configuration).await;
}
