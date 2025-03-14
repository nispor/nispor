// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_route::link::{InfoData, InfoVrf, InfoVrfPort};
use serde::{Deserialize, Serialize};

use crate::{ControllerType, Iface, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VrfInfo {
    pub table_id: u32,
    pub subordinates: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VrfSubordinateInfo {
    pub table_id: u32,
}

pub(crate) fn get_vrf_info(data: &InfoData) -> Option<VrfInfo> {
    if let InfoData::Vrf(infos) = data {
        let mut vrf_info = VrfInfo::default();
        for info in infos {
            if let InfoVrf::TableId(d) = *info {
                vrf_info.table_id = d;
            } else {
                log::debug!("Unknown VRF info {:?}", info)
            }
        }
        Some(vrf_info)
    } else {
        None
    }
}

pub(crate) fn get_vrf_subordinate_info(
    nlas: &[InfoVrfPort],
) -> Result<VrfSubordinateInfo, NisporError> {
    let mut ret = VrfSubordinateInfo::default();

    for nla in nlas {
        match nla {
            InfoVrfPort::TableId(d) => ret.table_id = *d,
            _ => {
                log::info!("Unknown VRF port info {:?}", nla);
            }
        }
    }
    Ok(ret)
}

pub(crate) fn vrf_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_subordinate_list_of_controller(iface_states);
}

fn gen_subordinate_list_of_controller(
    iface_states: &mut HashMap<String, Iface>,
) {
    let mut controller_subordinates: HashMap<String, Vec<String>> =
        HashMap::new();
    for iface in iface_states.values() {
        if iface.controller_type == Some(ControllerType::Vrf) {
            if let Some(controller) = &iface.controller {
                match controller_subordinates.get_mut(controller) {
                    Some(subordinates) => subordinates.push(iface.name.clone()),
                    None => {
                        let new_subordinates: Vec<String> =
                            vec![iface.name.clone()];
                        controller_subordinates
                            .insert(controller.clone(), new_subordinates);
                    }
                };
            }
        }
    }
    for (controller, subordinates) in controller_subordinates.iter_mut() {
        if let Some(controller_iface) = iface_states.get_mut(controller) {
            if let Some(ref mut vrf_info) = controller_iface.vrf {
                subordinates.sort();
                vrf_info.subordinates.clone_from(subordinates);
            }
        }
    }
}
