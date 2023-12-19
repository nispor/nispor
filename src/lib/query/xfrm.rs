// SPDX-License-Identifier: Apache-2.0
use std::collections::HashMap;

use netlink_packet_route::link::{InfoData, InfoXfrm};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct XfrmInfo {
    pub base_iface: String,
    pub iface_id: u32,
}

pub(crate) fn get_xfrm_info(data: &InfoData) -> Option<XfrmInfo> {
    if let InfoData::Xfrm(infos) = data {
        let mut xfrm_info = XfrmInfo::default();
        for info in infos {
            match *info {
                InfoXfrm::Link(i) => {
                    xfrm_info.base_iface = i.to_string();
                }
                InfoXfrm::IfId(d) => {
                    xfrm_info.iface_id = d;
                }
                _ => {
                    log::debug!("Unknown XFRM info {:?}", info);
                }
            }
        }
        Some(xfrm_info)
    } else {
        None
    }
}

pub(crate) fn xfrm_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    fill_port_iface_names(iface_states);
}

fn fill_port_iface_names(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(iface.index.to_string(), iface.name.clone());
    }
    for iface in iface_states
        .values_mut()
        .filter(|i| i.iface_type == IfaceType::Xfrm)
    {
        if let Some(xfrm_info) = iface.xfrm.as_mut() {
            if let Some(base_iface) = index_to_name.get(&xfrm_info.base_iface) {
                xfrm_info.base_iface = base_iface.to_string();
            }
        }
    }
}
