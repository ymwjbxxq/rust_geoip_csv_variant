use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CountryLocation {
    pub geoname_id: u64,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub locale_code: Option<String>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub continent_code: Option<String>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub continent_name: Option<String>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub country_iso_code: Option<String>,

    #[serde(deserialize_with = "csv::invalid_option")]
    pub country_name: Option<String>,

    #[serde(deserialize_with = "bool_from_string")]
    pub is_in_european_union: bool,
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
