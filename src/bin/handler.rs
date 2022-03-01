use aws_config::{RetryConfig, TimeoutConfig};
use geo_ip::{
    dtos::ip_request::IPRequest,
    queries::get_ip::{GetIP, GetIPQuery},
    utils::{api_helper::ApiHelper, ip_helper::IpAddrExt},
};
use lambda_http::{http::StatusCode, service_fn, Error, IntoResponse, Request};
use serde_json::json;
use std::{net::IpAddr, str::FromStr, time::Duration};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();
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
    lambda_http::run(service_fn(|event: Request| {
        execute(&dynamodb_client, event)
    }))
    .await?;
    Ok(())
}

pub async fn execute(client: &aws_sdk_dynamodb::Client, event: Request) -> Result<impl IntoResponse, Error> {
    println!("{:?}", &event);
    let ip_address = event.headers().get("x-forwarded-for");
    if let Some(ip_address) = ip_address {
        let ip_address = IpAddr::from_str(ip_address.to_str().unwrap()).unwrap();
        let request = IPRequest {
            ip_address,
            ip_address_decimal: ip_address.to_u64(),
        };

        let response = GetIP::new().await.execute(client, request).await?;
        if let Some(response) = response {
            return Ok(ApiHelper::response(
                StatusCode::OK,
                json!({"countrycode": response.country_code}).to_string(),
            ));
        }
    }
    Ok(ApiHelper::response(
        StatusCode::FORBIDDEN,
        json!({"message": "IP is not present in the header request"}).to_string(),
    ))
}
