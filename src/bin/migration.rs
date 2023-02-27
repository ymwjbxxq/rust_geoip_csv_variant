use futures::{try_join, StreamExt};
use geo_ip::dtos::country_location::CountryLocation;
use geo_ip::error::ApplicationError;
use geo_ip::models::geo_ip::GeoIP;
use geo_ip::modules::csv_parser::CSVParser;
use geo_ip::modules::sqs::Sqs;
use geo_ip::{dtos::country_blocks::CountryBlocks, modules::csv_parser::CSVBuilder};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let config = aws_config::load_from_env().await;
    let s3_client = aws_sdk_s3::Client::new(&config);
    let sqs_client = aws_sdk_sqs::Client::new(&config);

    run(service_fn(|_event: LambdaEvent<Value>| {
        handler(&s3_client, &sqs_client)
    }))
    .await
}

pub async fn handler(
    s3_client: &aws_sdk_s3::Client,
    sqs_client: &aws_sdk_sqs::Client,
) -> Result<(), ApplicationError> {
    let bucket_name = std::env::var("BUCKET_NAME").expect("BUCKET_NAME must be set");
    let ip_url = std::env::var("IP_URL").expect("IP_URL must be set");
    let location_url = std::env::var("LOCATION_URL").expect("LOCATION_URL must be set");

    let ip = CSVBuilder::new(s3_client)
        .bucket(bucket_name.to_string())
        .key(ip_url)
        .build();
    let ip_result = CSVParser::from_s3::<CountryBlocks>(ip);

    let country_location = CSVBuilder::new(s3_client)
        .bucket(bucket_name.to_string())
        .key(location_url)
        .build();
    let country_location_result = CSVParser::from_s3::<CountryLocation>(country_location);

    let result = try_join!(ip_result, country_location_result)?;

    let records = GeoIP::generate_records(result).await;

    let mut stream = tokio_stream::iter(records).chunks(1000);
    let sqs = Sqs::new(sqs_client);
    while let Some(v) = stream.next().await {
        sqs.send(v).await?;
    }

    Ok(())
}
