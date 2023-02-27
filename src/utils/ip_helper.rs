use std::net::IpAddr;
use std::str::FromStr;
pub struct IPHelper;

impl IPHelper {
    pub fn ip_to_bytes(address: IpAddr) -> Vec<u8> {
        match address {
            IpAddr::V4(a) => a.octets().to_vec(),
            IpAddr::V6(a) => a.octets().to_vec(),
        }
    }

    pub fn byte_to_u64(ip: Vec<u8>) -> u64 {
        let mut result: u64 = 0;
        for octet in ip.iter() {
            result = result * 1000 + *octet as u64;
        }
        result
    }

    pub fn is_first_last_same_first_octet(cidr: &str) -> bool {
        let first_octet_of_first_address = IPHelper::first_octet_of_first_address(cidr);
        let first_octet_of_last_address = IPHelper::first_octet_of_last_address(cidr);

        first_octet_of_first_address == first_octet_of_last_address
    }

    pub fn first_octet_of_first_address(cidr: &str) -> String {
        let cidr = cidr::IpCidr::from_str(cidr).unwrap();
        cidr.first_address().first_octet()
    }

    pub fn first_octet_of_last_address(cidr: &str) -> String {
        let cidr = cidr::IpCidr::from_str(cidr).unwrap();
        cidr.last_address().first_octet()
    }
}

pub trait IpAddrExt {
    fn first_octet(self) -> String;
    fn to_u64(self) -> u64;
}

impl IpAddrExt for IpAddr {
    fn first_octet(self) -> String {
        let ip_bytes = IPHelper::ip_to_bytes(self);
        let mut result: u64 = 0;
        if let Some(octet) = ip_bytes.first() {
            result = result * 1000 + *octet as u64;
        }
        result.to_string()
    }

    fn to_u64(self) -> u64 {
        let ip_to_bytes = IPHelper::ip_to_bytes(self);
        IPHelper::byte_to_u64(ip_to_bytes)
    }
}
