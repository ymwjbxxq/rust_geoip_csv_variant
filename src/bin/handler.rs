use geo_ip::{
    dtos::ip_request::IPRequest,
    queries::get_ip::{GetIP, GetIPQuery},
    utils::{api_helper::ApiHelper, ip_helper::IpAddrExt},
};
use lambda_http::{http::StatusCode, run, service_fn, Error, IntoResponse, Request};
use serde_json::json;
use std::{net::IpAddr, str::FromStr};

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing_subscriber::filter::LevelFilter::INFO)
        .init();

    let config = aws_config::load_from_env().await;
    let dynamodb_client = aws_sdk_dynamodb::Client::new(&config);

    run(service_fn(|event: Request| {
        function_handler(&dynamodb_client, event)
    }))
    .await
}

pub async fn function_handler(
    client: &aws_sdk_dynamodb::Client,
    event: Request,
) -> Result<impl IntoResponse, Error> {
    println!("{:?}", &event);
    let ip_address = event.headers().get("x-forwarded-for");
    if let Some(ip_address) = ip_address {
        let ip_address = IpAddr::from_str(ip_address.to_str()?).unwrap();
        let request = IPRequest {
            ip_address,
            ip_address_decimal: ip_address.to_u64(),
        };

        let country_code = GetIP::new(client)
            .await
            .country_code(request)
            .await
            .ok()
            .and_then(|response| response);
        if let Some(country_code) = country_code {
            return Ok(ApiHelper::response(
                StatusCode::OK,
                json!({ "countrycode": country_code }).to_string(),
            ));
        }
    }
    Ok(ApiHelper::response(
        StatusCode::FORBIDDEN,
        json!({"message": "IP is not present in the header request"}).to_string(),
    ))
}
