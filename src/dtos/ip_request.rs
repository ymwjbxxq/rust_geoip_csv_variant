use std::net::IpAddr;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct IPRequest {
    pub ip_address: IpAddr,
    pub ip_address_decimal: u64,
}