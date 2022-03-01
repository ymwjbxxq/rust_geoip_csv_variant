use aws_config::{RetryConfig, TimeoutConfig};
use aws_lambda_events::event::sqs::SqsEvent;
use futures::future::join_all;
use geo_ip::{models::geo_ip::GeoIP, queries::add_geo_ip::{AddGeoIP, AddGeoIPQuery}};
use lambda_runtime::{service_fn, Error, LambdaEvent};

use std::{sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    // Initialize AWS client
    let config = aws_config::from_env()
        .retry_config(RetryConfig::new().with_max_attempts(10))
        .timeout_config(
            TimeoutConfig::new()
                .with_read_timeout(Some(Duration::from_secs(1)))
                .with_connect_timeout(Some(Duration::from_secs(1)))
                .with_api_call_timeout(Some(Duration::from_secs(1))),
        )
        .load()
        .await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    lambda_runtime::run(service_fn(|event: LambdaEvent<SqsEvent>| {
        execute(&dynamodb_client, event)
    }))
    .await?;

    Ok(())
}

pub async fn execute(client: &aws_sdk_dynamodb::Client, event: LambdaEvent<SqsEvent>) -> Result<(), Error> {
    println!("{:?}", &event);
    let records = event.payload.records;

    let mut tasks = Vec::with_capacity(records.len());
    let shared_client = Arc::from(client.clone());
    for record in records.into_iter() {
        let shared_client = Arc::clone(&shared_client);
        tasks.push(tokio::spawn(async move {
            if let Some(body) = &record.body {
                let request: GeoIP = serde_json::from_str(&body).unwrap();
                AddGeoIP::new()
                    .await
                    .execute(&shared_client, request)
                    .await
                    .map_or_else(|e| println!("Error from add {:?}", e), |_| ());
            } else {
                print!("No body");
            }
        }))
    }

    join_all(tasks).await;

    Ok(())
}


