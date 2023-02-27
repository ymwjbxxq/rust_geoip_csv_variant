use aws_lambda_events::event::sqs::SqsEvent;
use futures::future::join_all;
use geo_ip::{
    models::geo_ip::GeoIP,
    queries::add_geo_ip::{AddGeoIP, AddGeoIPQuery},
};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    run(service_fn(|event: LambdaEvent<SqsEvent>| {
        handler(&dynamodb_client, event)
    }))
    .await
}

pub async fn handler(
    client: &aws_sdk_dynamodb::Client,
    event: LambdaEvent<SqsEvent>,
) -> Result<(), Error> {
    println!("{:?}", &event);
    let mut tasks = Vec::with_capacity(event.payload.records.len());
    let shared_client = Arc::from(client.clone());

    for record in event.payload.records.into_iter() {
        let shared_client = shared_client.clone();
        tasks.push(tokio::spawn(async move {
            if let Some(body) = &record.body {
                let request: GeoIP = serde_json::from_str(body).unwrap();
                AddGeoIP::new(&shared_client)
                    .await
                    .execute(&request)
                    .await
                    .map_or_else(|e| println!("Error from add {e:?}"), |_| ());
            }
        }));
    }

    join_all(tasks).await;

    Ok(())
}
