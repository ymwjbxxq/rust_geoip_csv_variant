use crate::{error::ApplicationError, models::geo_ip::GeoIP, utils::ip_helper::IPHelper};
use async_trait::async_trait;
use aws_sdk_dynamodb::{
    model::{AttributeValue, ReturnValue},
    Client,
};

#[async_trait]
pub trait AddGeoIPQuery {
    async fn new() -> Self;
    async fn execute(&self, client: &Client, request: GeoIP) -> Result<(), ApplicationError>;
    async fn add(&self, client: &aws_sdk_dynamodb::Client, pk: String, request: &GeoIP) -> Result<(), ApplicationError>;
}

#[derive(Debug)]
pub struct AddGeoIP {
    ip_v4_table_name: String,
    ip_v6_table_name: String,
}

#[async_trait]
impl AddGeoIPQuery for AddGeoIP {
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

    async fn execute(&self, client: &Client, request: GeoIP) -> Result<(), ApplicationError> {
        let pk = IPHelper::first_octet_of_first_address(&request.network);
        self.add(&client, pk, &request).await?;
        if IPHelper::is_first_last_same_first_octet(&request.network) {
            let pk = IPHelper::first_octet_of_last_address(&request.network);
            self.add(&client, pk, &request).await?;
        }

        Ok(())
    }

    async fn add(&self, client: &aws_sdk_dynamodb::Client, pk: String, request: &GeoIP) -> Result<(), ApplicationError> {
        let table_name;
        if request.is_ipv4 {
            table_name = self.ip_v4_table_name.clone();
        } else {
            table_name = self.ip_v6_table_name.clone();
        }
        client
            .update_item()
            .table_name(table_name)
            .key("pk", AttributeValue::S(pk))
            .key("sk", AttributeValue::S(request.network.to_string()))
            .update_expression("SET #min = :min_ip, #max = :max_ip, geoname_id = :geoname_id, is_anonymous_proxy = :is_anonymous_proxy, locale_code = :locale_code, continent_code = :continent_code, continent_name = :continent_name, country_iso_code = :country_iso_code, country_name = :country_name, is_in_european_union = :is_in_european_union")
            .expression_attribute_names("#min", "min")
            .expression_attribute_names("#max", "max")
            .expression_attribute_values(":min_ip",AttributeValue::N(format!("{:}", request.cidr_first_address.to_string())))
            .expression_attribute_values(":max_ip",AttributeValue::N(format!("{:}", request.cidr_last_address.to_string())))
            .expression_attribute_values(":geoname_id",AttributeValue::N(format!("{:}", request.geoname_id)))
            .expression_attribute_values(":is_anonymous_proxy",AttributeValue::S(format!("{:}", request.is_anonymous_proxy)))
            .expression_attribute_values(":locale_code",AttributeValue::S(format!("{:}", request.locale_code)))
            .expression_attribute_values(":continent_code",AttributeValue::S(format!("{:}", request.continent_code)))
            .expression_attribute_values(":continent_name",AttributeValue::S(format!("{:}", request.continent_name)))
            .expression_attribute_values(":country_iso_code",AttributeValue::S(format!("{:}", request.country_iso_code)))
            .expression_attribute_values(":country_name",AttributeValue::S(format!("{:}", request.country_name)))
            .expression_attribute_values(":is_in_european_union",AttributeValue::S(format!("{:}", request.is_in_european_union)))
            .return_values(ReturnValue::None)
            .send()
            .await?;

        Ok(())
    }
}
