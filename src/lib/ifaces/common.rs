use crate::ifaces::bond::BondConf;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IfaceType {
    Bond,
    Unknown,
    Other(String),
}

impl Default for IfaceType {
    fn default() -> Self {
        IfaceType::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IfaceState {
    Up,
    Down,
    Unknown,
}

impl Default for IfaceState {
    fn default() -> Self {
        IfaceState::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Iface {
    pub name: String,
    pub iface_type: IfaceType,
    pub state: IfaceState,
    pub mtu: i64,
    pub mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_conf: Option<BondConf>,
}
