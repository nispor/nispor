use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv4Info {
    pub addresses: Vec<Ipv4AddrInfo>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv4AddrInfo {
    pub address: String,
    pub prefix_len: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    // The renaming seonds for this address be valid
    pub valid_lft: String,
    // The renaming seonds for this address be preferred
    pub preferred_lft: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv6Info {
    pub addresses: Vec<Ipv6AddrInfo>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv6AddrInfo {
    pub address: String,
    pub prefix_len: u8,
    // The renaming seonds for this address be valid
    pub valid_lft: String,
    // The renaming seonds for this address be preferred
    pub preferred_lft: String,
}
