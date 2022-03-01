use crate::{
    dtos::{ip_request::IPRequest, ip_response::IPResponse},
    error::ApplicationError,
    utils::{dynamodb::AttributeValuesExt, ip_helper::IpAddrExt},
};
use as_num::AsNum;
use async_trait::async_trait;
use aws_sdk_dynamodb::{model::AttributeValue, Client};

#[async_trait]
pub trait GetIPQuery {
    async fn new() -> Self;
    async fn execute(&self, client: &Client, request: IPRequest) -> Result<Option<IPResponse>, ApplicationError>;
}

#[derive(Debug)]
pub struct GetIP {
    ip_v4_table_name: String,
    ip_v6_table_name: String,
}

#[async_trait]
impl GetIPQuery for GetIP {
    async fn new() -> Self {
        let ip_v4_table_name =
            std::env::var("IPv4_TABLE_NAME").expect("IPv4_TABLE_NAME must be set");
        let ip_v6_table_name =
            std::env::var("IPv6_TABLE_NAME").expect("IPv6_TABLE_NAME must be set");
        Self {
            ip_v4_table_name,
            ip_v6_table_name,
        }
    }

    async fn execute(&self, client: &Client, request: IPRequest) -> Result<Option<IPResponse>, ApplicationError> {
        let table_name;
        if request.ip_address.is_ipv4() {
            table_name = self.ip_v4_table_name.clone();
        } else {
            table_name = self.ip_v6_table_name.clone();
        }

        let results = client
            .query()
            .table_name(table_name)
            .key_condition_expression("pk = :pk ")
            .expression_attribute_values(":pk", AttributeValue::S(request.ip_address.first_octet()))
            .send()
            .await;

        if let Ok(results) = results {
            if let Some(items) = results.items {
                let items: Vec<IPResponse> = items
                    .into_iter()
                    .filter(|row| {
                        request.ip_address_decimal >= row.get_number("min").unwrap().as_num()
                            && request.ip_address_decimal <= row.get_number("max").unwrap().as_num()
                    })
                    .map(|row| {
                        println!("row {:?}", row);
                        let response = IPResponse {
                            country_code: row.get_string("country_iso_code").unwrap(),
                        };
                        response
                    })
                    .collect();

                return Ok(Some(items.first().unwrap().clone()));
            }
        }

        Ok(None)
    }
}
