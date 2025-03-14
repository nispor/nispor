// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_route::link::{InfoData, InfoIpVlan, IpVlanFlags};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

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

impl From<rtnetlink::packet_route::link::IpVlanMode> for IpVlanMode {
    fn from(d: rtnetlink::packet_route::link::IpVlanMode) -> Self {
        match d {
            rtnetlink::packet_route::link::IpVlanMode::L2 => Self::L2,
            rtnetlink::packet_route::link::IpVlanMode::L3 => Self::L3,
            rtnetlink::packet_route::link::IpVlanMode::L3S => Self::L3S,
            _ => Self::Other(d.into()),
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

impl From<IpVlanFlags> for IpVlanFlag {
    fn from(d: IpVlanFlags) -> Self {
        match d {
            IpVlanFlags::Private => Self::Private,
            IpVlanFlags::Vepa => Self::Vepa,
            _ => Self::Other(d.bits()),
        }
    }
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
                InfoIpVlan::Flags(d) => {
                    ipv_info.flags = d.iter().map(IpVlanFlag::from).collect()
                }
                _ => log::debug!("Unkwnown IP VLAN info {:?}", info),
            }
        }
        return Some(ipv_info);
    }
    None
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
