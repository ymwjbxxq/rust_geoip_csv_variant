use futures::{try_join, StreamExt};
use geo_ip::aws::client::{AWSClient, AWSConfig};
use geo_ip::dtos::country_blocks::CountryBlocks;
use geo_ip::dtos::country_location::CountryLocation;
use geo_ip::error::ApplicationError;
use geo_ip::models::geo_ip::GeoIP;
use geo_ip::modules::csv_parser::{CSVBuilder, CSVParser};
use geo_ip::modules::sqs::Sqs;
use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::Value;

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
    let config = aws_config::load_from_env().await;
    let config = AWSConfig::set_config(config);
    let aws_client = config.migration_init();

    lambda_runtime::run(service_fn(|event: LambdaEvent<Value>| {
        execute(&aws_client, event)
    }))
    .await?;

    Ok(())
}

pub async fn execute(aws_client: &AWSClient, _event: LambdaEvent<Value>) -> Result<(), ApplicationError> {
    let bucket_name = std::env::var("BUCKET_NAME").expect("BUCKET_NAME must be set");
    let ip_url = std::env::var("IP_URL").expect("IP_URL must be set");
    let location_url = std::env::var("LOCATION_URL").expect("LOCATION_URL must be set");

    let ip = CSVBuilder::new(&aws_client.s3_client)
        .bucket(bucket_name.to_string())
        .key(ip_url)
        .build();
    let ip_result = CSVParser::from_s3::<CountryBlocks>(ip);

    let country_location = CSVBuilder::new(&aws_client.s3_client)
        .bucket(bucket_name.to_string())
        .key(location_url)
        .build();
    let country_location_result = CSVParser::from_s3::<CountryLocation>(country_location);

    let result = try_join!(
        ip_result,
        country_location_result
    )?;

    let records = GeoIP::generate_records(result).await;

    let mut stream = tokio_stream::iter(records).chunks(1000);
    while let Some(v) = stream.next().await {
        Sqs::send(&aws_client.sqs_client, v).await?;
    }
    
    Ok(())
}