// SPDX-License-Identifier: Apache-2.0

use netlink_packet_route::rtnl::link::nlas;
use serde::{Deserialize, Serialize};

use super::mac_vlan::get_mac_vlan_info;
use crate::{MacVlanInfo, MacVlanMode, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
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

pub(crate) fn get_mac_vtap_info(
    data: &nlas::InfoData,
) -> Result<Option<MacVtapInfo>, NisporError> {
    if let Some(info) = get_mac_vlan_info(data)? {
        Ok(Some(info.into()))
    } else {
        Ok(None)
    }
}
