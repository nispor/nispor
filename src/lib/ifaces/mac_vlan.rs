use crate::mac::parse_as_mac;
use crate::netlink::parse_as_u16;
use crate::netlink::parse_as_u32;
use crate::Iface;
use crate::IfaceType;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

const ETH_ALEN: usize = 6;

const MACVLAN_MODE_PRIVATE: u32 = 1;
const MACVLAN_MODE_VEPA: u32 = 2;
const MACVLAN_MODE_BRIDGE: u32 = 4;
const MACVLAN_MODE_PASSTHRU: u32 = 8;
const MACVLAN_MODE_SOURCE: u32 = 16;

const IFLA_MACVLAN_MODE: u16 = 1;
const IFLA_MACVLAN_FLAGS: u16 = 2;
// const IFLA_MACVLAN_MACADDR_MODE: u16 = 3;    // not used in query
const IFLA_MACVLAN_MACADDR: u16 = 4;
const IFLA_MACVLAN_MACADDR_DATA: u16 = 5;
const IFLA_MACVLAN_MACADDR_COUNT: u16 = 6;

#[serde(rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MacVlanMode {
    /* don't talk to other macvlans */
    Private,
    /* talk to other ports through ext bridge */
    Vepa,
    /* talk to bridge ports directly */
    Bridge,
    /* take over the underlying device */
    #[serde(rename = "passthru")]
    PassThrough,
    /* use source MAC address list to assign */
    Source,
    Other(u32),
    Unknown,
}

impl Default for MacVlanMode {
    fn default() -> Self {
        MacVlanMode::Unknown
    }
}

impl From<u32> for MacVlanMode {
    fn from(d: u32) -> Self {
        match d {
            MACVLAN_MODE_PRIVATE => Self::Private,
            MACVLAN_MODE_VEPA => Self::Vepa,
            MACVLAN_MODE_BRIDGE => Self::Bridge,
            MACVLAN_MODE_PASSTHRU => Self::PassThrough,
            MACVLAN_MODE_SOURCE => Self::Source,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct MacVlanInfo {
    pub base_iface: String,
    pub mode: MacVlanMode,
    pub flags: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mac_addresses: Option<Vec<String>>,
}

pub(crate) fn get_mac_vlan_info(data: &nlas::InfoData) -> Option<MacVlanInfo> {
    let mut info = MacVlanInfo::default();
    if let nlas::InfoData::MacVlan(raw) | nlas::InfoData::MacVtap(raw)= data {
        let nlas = NlasIterator::new(raw);
        for nla in nlas {
            match nla {
                Ok(nla) => match nla.kind() {
                    IFLA_MACVLAN_MODE => {
                        info.mode = parse_as_u32(nla.value()).into();
                    }
                    IFLA_MACVLAN_FLAGS => {
                        info.flags = parse_as_u16(nla.value());
                    }
                    IFLA_MACVLAN_MACADDR_COUNT => {
                        // Ignore. Use info.allowed_mac_addresses.len()
                        ()
                    }
                    IFLA_MACVLAN_MACADDR_DATA => {
                        info.allowed_mac_addresses =
                            Some(parse_mac_addr_data(nla.value()));
                    }
                    _ => {
                        eprintln!(
                            "Unhandled MAC VLAN IFLA_INFO_DATA: {} {:?}",
                            nla.kind(),
                            nla.value()
                        );
                    }
                },
                Err(e) => {
                    eprintln!(
                        "MAC VLAN IFLA_INFO_DATA NlasIterator failure: {}",
                        e
                    );
                }
            }
        }
        Some(info)
    } else {
        None
    }
}

fn parse_mac_addr_data(raw: &[u8]) -> Vec<String> {
    let mut addrs = Vec::new();
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_MACVLAN_MACADDR => {
                    addrs.push(parse_as_mac(ETH_ALEN, nla.value()));
                }
                _ => {
                    eprintln!(
                        "Unhanlded IFLA_MACVLAN_MACADDR_DATA: {} {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "IFLA_MACVLAN_MACADDR_DATA NlasIterator failure: {}",
                    e
                );
            }
        }
    }
    addrs
}

pub(crate) fn mac_vlan_iface_tidy_up(
    iface_states: &mut HashMap<String, Iface>,
) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::MacVlan &&
           iface.iface_type != IfaceType::MacVtap
        {
            continue;
        }
        if let Some(ref mut info) = iface.mac_vlan {
            if let Some(base_iface_name) = index_to_name.get(&info.base_iface) {
                info.base_iface = base_iface_name.clone();
            }
        } else if let Some(ref mut info) = iface.mac_vtap {
            if let Some(base_iface_name) = index_to_name.get(&info.base_iface) {
                info.base_iface = base_iface_name.clone();
            }
        }
    }
}
