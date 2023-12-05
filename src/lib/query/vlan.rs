// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{self, InfoData, InfoVlan};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum VlanProtocol {
    #[serde(rename = "802.1q")]
    #[default]
    Ieee8021Q,
    #[serde(rename = "802.1ad")]
    Ieee8021AD,
    Unknown,
}

impl From<link::VlanProtocol> for VlanProtocol {
    fn from(d: link::VlanProtocol) -> Self {
        match d {
            link::VlanProtocol::Ieee8021Q => Self::Ieee8021Q,
            link::VlanProtocol::Ieee8021Ad => Self::Ieee8021AD,
            _ => Self::Unknown,
        }
    }
}

const VLAN_FLAG_REORDER_HDR: u32 = 0x1;
const VLAN_FLAG_GVRP: u32 = 0x2;
const VLAN_FLAG_LOOSE_BINDING: u32 = 0x4;
const VLAN_FLAG_MVRP: u32 = 0x8;
const VLAN_FLAG_BRIDGE_BINDING: u32 = 0x10;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VlanInfo {
    pub vlan_id: u16,
    pub protocol: VlanProtocol,
    pub base_iface: String,
    pub is_reorder_hdr: bool,
    pub is_gvrp: bool,
    pub is_loose_binding: bool,
    pub is_mvrp: bool,
    pub is_bridge_binding: bool,
}

pub(crate) fn get_vlan_info(data: &InfoData) -> Option<VlanInfo> {
    if let InfoData::Vlan(infos) = data {
        let mut vlan_info = VlanInfo::default();
        for info in infos {
            if let InfoVlan::Id(d) = info {
                vlan_info.vlan_id = *d;
            } else if let InfoVlan::Protocol(d) = info {
                vlan_info.protocol = (*d).into();
            } else if let InfoVlan::Flags((flags, _)) = info {
                // The kernel always set the mask as u32::MAX
                if *flags & VLAN_FLAG_REORDER_HDR > 0 {
                    vlan_info.is_reorder_hdr = true
                }
                if *flags & VLAN_FLAG_GVRP > 0 {
                    vlan_info.is_gvrp = true
                }
                if *flags & VLAN_FLAG_LOOSE_BINDING > 0 {
                    vlan_info.is_loose_binding = true
                }
                if *flags & VLAN_FLAG_MVRP > 0 {
                    vlan_info.is_mvrp = true
                }
                if *flags & VLAN_FLAG_BRIDGE_BINDING > 0 {
                    vlan_info.is_bridge_binding = true
                }
            } else {
                log::debug!("Unknown VLAN info: {:?}", info);
            }
        }
        Some(vlan_info)
    } else {
        None
    }
}

pub(crate) fn vlan_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Vlan {
            continue;
        }
        if let Some(ref mut vlan_info) = iface.vlan {
            if let Some(base_iface_name) =
                index_to_name.get(&vlan_info.base_iface)
            {
                vlan_info.base_iface = base_iface_name.clone();
            }
        }
    }
}
