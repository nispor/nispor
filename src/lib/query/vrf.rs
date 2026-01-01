// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_route::link::{InfoData, InfoVrf, InfoVrfPort};
use serde::{Deserialize, Serialize};

use crate::{ControllerType, Iface, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VrfInfo {
    pub table_id: u32,
    pub ports: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VrfPortInfo {
    pub table_id: u32,
}

pub(crate) fn get_vrf_info(data: &InfoData) -> Option<VrfInfo> {
    if let InfoData::Vrf(infos) = data {
        let mut vrf_info = VrfInfo::default();
        for info in infos {
            if let InfoVrf::TableId(d) = *info {
                vrf_info.table_id = d;
            } else {
                log::debug!("Unknown VRF info {info:?}")
            }
        }
        Some(vrf_info)
    } else {
        None
    }
}

pub(crate) fn get_vrf_port_info(
    nlas: &[InfoVrfPort],
) -> Result<VrfPortInfo, NisporError> {
    let mut ret = VrfPortInfo::default();

    for nla in nlas {
        match nla {
            InfoVrfPort::TableId(d) => ret.table_id = *d,
            _ => {
                log::info!("Unknown VRF port info {nla:?}");
            }
        }
    }
    Ok(ret)
}

pub(crate) fn vrf_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_port_list_of_controller(iface_states);
}

fn gen_port_list_of_controller(iface_states: &mut HashMap<String, Iface>) {
    let mut controller_ports: HashMap<String, Vec<String>> = HashMap::new();
    for iface in iface_states.values() {
        if iface.controller_type == Some(ControllerType::Vrf) {
            if let Some(controller) = &iface.controller {
                match controller_ports.get_mut(controller) {
                    Some(ports) => ports.push(iface.name.clone()),
                    None => {
                        let new_ports: Vec<String> = vec![iface.name.clone()];
                        controller_ports.insert(controller.clone(), new_ports);
                    }
                };
            }
        }
    }
    for (controller, ports) in controller_ports.iter_mut() {
        if let Some(controller_iface) = iface_states.get_mut(controller) {
            if let Some(ref mut vrf_info) = controller_iface.vrf {
                ports.sort();
                vrf_info.ports.clone_from(ports);
            }
        }
    }
}
