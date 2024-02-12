use anyhow::Context;
use aws_sdk_sqs::Client;
use uuid::Uuid;

use domain::prelude::{
    CreateSubscriber, SubscriberCommand, SubscriberCommandExecutor, SubscriberRepository,
};

#[derive(Debug, serde::Deserialize)]
pub struct Request {
    pub id: Uuid,
    pub email: String,
    pub name: String,
}

async fn handle(
    client: Client,
    url: &str,
    command_executor: SubscriberCommandExecutor<impl SubscriberRepository>,
) -> Result<(), anyhow::Error> {
    let received_messages = client
        .receive_message()
        .queue_url(url)
        .send()
        .await
        .context(format!("Failed to receive messages from SQS {}", url))?;

    for message in received_messages.messages.unwrap_or_default() {
        let request: Request = serde_json::from_str(message.body.unwrap().as_str()).unwrap();

        let command = SubscriberCommand::CreateSubscriber(CreateSubscriber::new(
            request.id,
            request.email,
            request.name,
        ));
        command_executor.execute(command).await.unwrap();

        if let Some(receipt_handle) = message.receipt_handle {
            client
                .delete_message()
                .queue_url(url)
                .receipt_handle(receipt_handle)
                .send()
                .await
                .unwrap();
        }
    }

    Ok(())
}

