use crate::ifaces::bond::BondInfo;
use crate::ifaces::bond::BondSlaveInfo;
use crate::Ipv4Info;
use crate::Ipv6Info;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum IfaceType {
    Bond,
    Veth,
    Bridge,
    Vlan,
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MasterType {
    Bond,
    Unknown,
}

impl From<&str> for MasterType {
    fn from(s: &str) -> Self {
        match s {
            "bond" => MasterType::Bond,
            _ => MasterType::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Iface {
    pub name: String,
    //    #[serde(skip_serializing)]
    pub index: u32,
    pub iface_type: IfaceType,
    pub state: IfaceState,
    pub mtu: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<Ipv4Info>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<Ipv6Info>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_info: Option<BondInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_type: Option<MasterType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_slave_info: Option<BondSlaveInfo>,
}

pub(crate) fn get_iface_name_by_index(
    iface_states: &HashMap<String, Iface>,
    iface_index: u32,
) -> String {
    for (iface_name, iface) in iface_states.iter() {
        if iface.index == iface_index {
            return iface_name.clone();
        }
    }
    "".into()
}
