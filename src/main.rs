use newsletter::api;
use newsletter::configuration;

#[tokio::main]
async fn main() {
    let configuration = configuration::get_configuration().await;
    let listener = configuration::get_listener(&configuration).await;

    api::runner::run(listener).await
}
