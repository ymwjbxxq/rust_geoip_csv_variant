use crate::{error::ApplicationError, models::geo_ip::GeoIP};
use aws_sdk_sqs::{model::SendMessageBatchRequestEntry, Client};
use futures::future::join_all;
use std::sync::Arc;

#[derive(Debug)]
pub struct Sqs {
    client: Client,
}

impl Sqs {
    pub fn new(client: &Client) -> Sqs {
        Self {
            client: client.clone(),
        }
    }

    pub async fn send(&self, records: Vec<GeoIP>) -> Result<(), ApplicationError> {
        let mut tasks = Vec::with_capacity(records.len());
        let shared_client = Arc::from(self.client.clone());
        records.chunks(10).for_each(|chunk| {
            let shared_client = Arc::clone(&shared_client);
            let entries = chunk
                .iter()
                .map(|record| {
                    SendMessageBatchRequestEntry::builder()
                        .id(uuid::Uuid::new_v4().to_string())
                        .message_body(serde_json::to_string(&record).unwrap())
                        .build()
                })
                .collect::<Vec<SendMessageBatchRequestEntry>>();

            tasks.push(tokio::spawn(async move {
                send_batch(&shared_client, entries)
                    .await
                    .map_or_else(|e| println!("Error from send_to_sqs {e:?}"), |_| ());
            }));
        });

        join_all(tasks).await;

        Ok(())
    }
}

async fn send_batch(
    client: &Client,
    entries: Vec<SendMessageBatchRequestEntry>,
) -> Result<(), ApplicationError> {
    // send to sqs
    let sqs_url = std::env::var("SQS_URL").expect("SQS_URL must be set");
    client
        .send_message_batch()
        .queue_url(sqs_url)
        .set_entries(Some(entries))
        .send()
        .await?;
    Ok(())
}
