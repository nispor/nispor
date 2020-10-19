use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

pub(crate) fn parse_as_u8(data: &[u8]) -> u8 {
    data[0]
}

pub(crate) fn parse_as_u16(data: &[u8]) -> u16 {
    u16::from_ne_bytes([data[0], data[1]])
}

pub(crate) fn parse_as_be16(data: &[u8]) -> u16 {
    u16::from_be_bytes([data[0], data[1]])
}

pub(crate) fn parse_as_i32(data: &[u8]) -> i32 {
    i32::from_ne_bytes([data[0], data[1], data[2], data[3]])
}

pub(crate) fn parse_as_u32(data: &[u8]) -> u32 {
    u32::from_ne_bytes([data[0], data[1], data[2], data[3]])
}

pub(crate) fn parse_as_be32(data: &[u8]) -> u32 {
    u32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

pub(crate) fn parse_as_u64(data: &[u8]) -> u64 {
    u64::from_ne_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ])
}

pub(crate) fn parse_as_ipv4(data: &[u8]) -> String {
    Ipv4Addr::from([data[0], data[1], data[2], data[3]]).to_string()
}

pub(crate) fn parse_as_ipv6(data: &[u8]) -> String {
    let mut addr_bytes = [0u8; 16];
    addr_bytes.copy_from_slice(&data[..16]);
    Ipv6Addr::from(addr_bytes).to_string()
}
