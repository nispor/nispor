use crate::ifaces::mac_vlan::get_mac_vlan_info;
use crate::ifaces::mac_vlan::MacVlanInfo;
use crate::ifaces::mac_vlan::MacVlanMode;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};

#[serde(rename_all = "lowercase")]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MacVtapMode {
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

impl Default for MacVtapMode {
    fn default() -> Self {
        MacVtapMode::Unknown
    }
}

impl From<MacVlanMode> for MacVtapMode {
    fn from(d: MacVlanMode) -> Self {
        match d {
            MacVlanMode::Private => Self::Private,
            MacVlanMode::Vepa => Self::Vepa,
            MacVlanMode::Bridge => Self::Bridge,
            MacVlanMode::PassThrough => Self::PassThrough,
            MacVlanMode::Source => Self::Source,
            MacVlanMode::Unknown => Self::Unknown,
            MacVlanMode::Other(u32) => Self::Other(u32),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct MacVtapInfo {
    pub base_iface: String,
    pub mode: MacVtapMode,
    pub flags: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_mac_addresses: Option<Vec<String>>,
}

impl From<MacVlanInfo> for MacVtapInfo {
    fn from(d: MacVlanInfo) -> Self {
        Self {
            base_iface: d.base_iface,
            mode: MacVtapMode::from(d.mode),
            flags: d.flags,
            allowed_mac_addresses: d.allowed_mac_addresses,
        }
    }
}
pub(crate) fn get_mac_vtap_info(data: &nlas::InfoData) -> Option<MacVtapInfo> {
    if let Some(info) = get_mac_vlan_info(data) {
        Some(info.into())
    } else {
        None
    }
}
