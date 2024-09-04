// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{InfoData, InfoIpVlan};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

const IPVLAN_MODE_L2: u16 = 0;
const IPVLAN_MODE_L3: u16 = 1;
const IPVLAN_MODE_L3S: u16 = 2;

const IPVLAN_F_PRIVATE: u16 = 0x01;
const IPVLAN_F_VEPA: u16 = 0x02;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IpVlanMode {
    L2,
    L3,
    L3S,
    Other(u16),
}

impl Default for IpVlanMode {
    fn default() -> Self {
        IpVlanMode::L3
    }
}

impl From<u16> for IpVlanMode {
    fn from(d: u16) -> Self {
        match d {
            IPVLAN_MODE_L2 => Self::L2,
            IPVLAN_MODE_L3 => Self::L3,
            IPVLAN_MODE_L3S => Self::L3S,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum IpVlanFlag {
    Private,
    Vepa,
    Other(u16),
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct IpVlanInfo {
    pub base_iface: String,
    pub mode: IpVlanMode,
    pub flags: Vec<IpVlanFlag>,
}

pub(crate) fn get_ip_vlan_info(data: &InfoData) -> Option<IpVlanInfo> {
    let mut ipv_info = IpVlanInfo::default();
    if let InfoData::IpVlan(infos) = data {
        for info in infos {
            match *info {
                InfoIpVlan::Mode(d) => ipv_info.mode = d.into(),
                InfoIpVlan::Flags(d) => ipv_info.flags = ipvlan_flag_to_enum(d),
                _ => log::debug!("Unkwnown IP VLAN info {:?}", info),
            }
        }
        return Some(ipv_info);
    }
    None
}

fn ipvlan_flag_to_enum(flags: u16) -> Vec<IpVlanFlag> {
    let mut flags_vec = Vec::new();

    if (IPVLAN_F_PRIVATE & flags) == IPVLAN_F_PRIVATE {
        flags_vec.push(IpVlanFlag::Private);
    }
    if (IPVLAN_F_VEPA & flags) == IPVLAN_F_VEPA {
        flags_vec.push(IpVlanFlag::Vepa);
    }

    flags_vec
}

pub(crate) fn ip_vlan_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::IpVlan {
            continue;
        }
        if let Some(ref mut info) = iface.ip_vlan {
            if let Some(base_iface_name) = index_to_name.get(&info.base_iface) {
                info.base_iface.clone_from(base_iface_name);
            }
        }
    }
}
