use csv;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountryBlocks {
    pub network: String,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub geoname_id: Option<u64>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub registered_country_geoname_id: Option<u64>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub represented_country_geoname_id: Option<u64>,

    #[serde(deserialize_with = "bool_from_string")]
    pub is_anonymous_proxy: bool,
}

/// Deserialize bool from String with custom value mapping
fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?.as_ref() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Ok(false),
    }
}
