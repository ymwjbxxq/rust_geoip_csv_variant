use crate::{
    dtos::{country_blocks::CountryBlocks, country_location::CountryLocation},
    utils::ip_helper::IpAddrExt,
};
use futures::StreamExt;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr, sync::Arc};

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct GeoIP {
    pub network: String,
    pub geoname_id: u64,
    pub registered_country_geoname_id: u64,
    pub represented_country_geoname_id: u64,
    pub is_anonymous_proxy: bool,
    pub locale_code: String,
    pub continent_code: String,
    pub continent_name: String,
    pub country_iso_code: String,
    pub country_name: String,
    pub is_in_european_union: bool,
    pub cidr_first_address: u64,
    pub cidr_last_address: u64,
    pub is_ipv4: bool,
}

impl GeoIP {
    pub async fn generate_records(
        result: (Vec<CountryBlocks>, Vec<CountryLocation>),
    ) -> Vec<GeoIP> {
        let country_blocks = result.0;
        let country_locations = location_hashmap(result.1);
        println!("IP detected {:?}", country_blocks.len());

        let mut records: Vec<GeoIP> = Vec::with_capacity(country_blocks.len());
        let country_locations = Arc::from(country_locations);

        let mut stream = tokio_stream::iter(country_blocks);
        while let Some(country_block) = stream.next().await {
            let country_locations = Arc::clone(&country_locations);
            let item = generate_ip(&country_block, country_locations);

            records.push(item);
        }

        records
    }
}

fn location_hashmap(my_array: Vec<CountryLocation>) -> HashMap<u64, CountryLocation> {
    my_array
        .par_iter()
        .map(|row| {
            let mut m = HashMap::new();
            m.insert(row.geoname_id, row.clone());
            m
        })
        .flatten_iter()
        .collect()
}

fn generate_ip(
    country_block: &CountryBlocks,
    country_locations: Arc<HashMap<u64, CountryLocation>>,
) -> GeoIP {
    let geoname_id = get_geoname_id(country_block);
    let location = get_location(geoname_id, country_locations);
    GeoIPBuilder::new(country_block.network.to_string(), geoname_id)
        .registered_country_geoname_id(country_block.registered_country_geoname_id)
        .represented_country_geoname_id(country_block.represented_country_geoname_id)
        .is_anonymous_proxy(country_block.is_anonymous_proxy)
        .locale_code(location.locale_code)
        .continent_code(location.continent_code)
        .continent_name(location.continent_name)
        .country_iso_code(location.country_iso_code)
        .country_name(location.country_name)
        .is_in_european_union(location.is_in_european_union)
        .build()
}

fn get_location(
    geoname_id: u64,
    country_locations: Arc<HashMap<u64, CountryLocation>>,
) -> CountryLocation {
    let location = country_locations.get(&geoname_id);
    match location {
        Some(location) => location.clone(),
        None => CountryLocation {
            geoname_id,
            locale_code: None,
            continent_code: None,
            continent_name: None,
            country_iso_code: None,
            country_name: None,
            is_in_european_union: false,
        },
    }
}

fn get_geoname_id(country_block: &CountryBlocks) -> u64 {
    if country_block.geoname_id.is_some() {
        country_block.geoname_id.unwrap()
    } else {
        country_block.registered_country_geoname_id.unwrap()
    }
}

#[derive(Debug)]
pub struct GeoIPBuilder {
    network: String,
    geoname_id: u64,
    registered_country_geoname_id: Option<u64>,
    represented_country_geoname_id: Option<u64>,
    is_anonymous_proxy: bool,
    locale_code: Option<String>,
    continent_code: Option<String>,
    continent_name: Option<String>,
    country_iso_code: Option<String>,
    country_name: Option<String>,
    is_in_european_union: bool,
    cidr_first_address: u64,
    cidr_last_address: u64,
    is_ipv4: bool,
}

impl GeoIPBuilder {
    pub fn new(network: String, geoname_id: u64) -> GeoIPBuilder {
        let cidr = cidr::IpCidr::from_str(&network);
        if let Ok(cidr) = cidr {
            let cidr_first_address = cidr.first_address().to_u64();
            let cidr_last_address = cidr.last_address().to_u64();
            let is_ipv4 = cidr.is_ipv4();

            GeoIPBuilder {
                network,
                geoname_id,
                registered_country_geoname_id: None,
                represented_country_geoname_id: None,
                is_anonymous_proxy: false,
                locale_code: None,
                continent_code: None,
                continent_name: None,
                country_iso_code: None,
                country_name: None,
                is_in_european_union: false,
                cidr_first_address,
                cidr_last_address,
                is_ipv4,
            }
        } else {
            GeoIPBuilder {
                network,
                geoname_id,
                registered_country_geoname_id: None,
                represented_country_geoname_id: None,
                is_anonymous_proxy: false,
                locale_code: None,
                continent_code: None,
                continent_name: None,
                country_iso_code: None,
                country_name: None,
                is_in_european_union: false,
                cidr_first_address: 0,
                cidr_last_address: 0,
                is_ipv4: false,
            }
        }
    }

    pub fn registered_country_geoname_id(mut self, input: Option<u64>) -> GeoIPBuilder {
        self.registered_country_geoname_id = input;
        self
    }

    pub fn represented_country_geoname_id(mut self, input: Option<u64>) -> GeoIPBuilder {
        self.represented_country_geoname_id = input;
        self
    }

    pub fn is_anonymous_proxy(mut self, input: bool) -> GeoIPBuilder {
        self.is_anonymous_proxy = input;
        self
    }

    pub fn locale_code(mut self, input: Option<String>) -> GeoIPBuilder {
        self.locale_code = input;
        self
    }

    pub fn continent_code(mut self, input: Option<String>) -> GeoIPBuilder {
        self.continent_code = input;
        self
    }

    pub fn continent_name(mut self, input: Option<String>) -> GeoIPBuilder {
        self.continent_name = input;
        self
    }

    pub fn country_iso_code(mut self, input: Option<String>) -> GeoIPBuilder {
        self.country_iso_code = input;
        self
    }

    pub fn country_name(mut self, input: Option<String>) -> GeoIPBuilder {
        self.country_name = input;
        self
    }

    pub fn is_in_european_union(mut self, input: bool) -> GeoIPBuilder {
        self.is_in_european_union = input;
        self
    }

    pub fn build(self) -> GeoIP {
        GeoIP {
            network: self.network,
            geoname_id: self.geoname_id,
            cidr_first_address: self.cidr_first_address,
            cidr_last_address: self.cidr_last_address,
            is_ipv4: self.is_ipv4,
            registered_country_geoname_id: self.registered_country_geoname_id.unwrap_or(0),
            represented_country_geoname_id: self.represented_country_geoname_id.unwrap_or(0),
            is_anonymous_proxy: self.is_anonymous_proxy,
            locale_code: self.locale_code.unwrap_or("".to_string()),
            continent_code: self.continent_code.unwrap_or("".to_string()),
            continent_name: self.continent_name.unwrap_or("".to_string()),
            country_iso_code: self.country_iso_code.unwrap_or("".to_string()),
            country_name: self.country_name.unwrap_or("".to_string()),
            is_in_european_union: self.is_in_european_union,
        }
    }
}
