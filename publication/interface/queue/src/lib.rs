mod listeners;
pub mod runner;

use aws_config::BehaviorVersion;
use aws_sdk_sqs::Client;

async fn listener(client: Client, url: &str) -> Result<(), anyhow::Error> {
    let messages = client.receive_message().queue_url(url).send().await?;
    println!("{:?}", messages);

    for message in messages.messages.unwrap_or_default() {
        println!("{:?}", message);
    }

    Ok(())
}

// export AWS_ACCESS_KEY_ID="test"
// export AWS_SECRET_ACCESS_KEY="test"
// export AWS_DEFAULT_REGION="ap-northeast-2"
// aws sqs create-queue --queue-name example --region ap-northeast-2 --endpoint-url http://localhost:4566

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let endpoint_url = "http://localhost:4566/";
    let mut config = aws_config::defaults(BehaviorVersion::latest());
    config = config.endpoint_url(endpoint_url);

    let config_builder = aws_sdk_sqs::config::Builder::from(&config.load().await);
    let client = aws_sdk_sqs::Client::from_conf(config_builder.build());

    let queue_url =
        "http://sqs.ap-northeast-2.localhost.localstack.cloud:4566/000000000000/example";

    // let message = Message {
    //     title: "Hi".to_owned(),
    //     content: "There?".to_owned(),
    // };
    // client
    //     .send_message()
    //     .queue_url(queue_url)
    //     .message_body(&message.content)
    //     .send()
    //     .await
    //     .unwrap();

    let response = client
        .receive_message()
        .queue_url(queue_url)
        .send()
        .await
        .unwrap();

    println!("{:?}", response);

    for message in response.messages.unwrap_or_default() {
        println!("{:?}", message);

        if let Some(receipt_handle) = message.receipt_handle {
            client
                .delete_message()
                .queue_url(queue_url)
                .receipt_handle(receipt_handle)
                .send()
                .await
                .unwrap();
        }
    }

    Ok(())
}

struct Message {
    title: String,
    content: String,
}
