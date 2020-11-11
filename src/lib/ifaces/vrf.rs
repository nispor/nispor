use crate::netlink::parse_as_u32;
use crate::ControllerType;
use crate::Iface;
use crate::NisporError;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::nlas::NlaBuffer;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

const IFLA_VRF_PORT_TABLE: u16 = 1;
const IFLA_VRF_TABLE: u16 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct VrfInfo {
    pub table_id: u32,
    pub subordinates: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct VrfSubordinateInfo {
    pub table_id: u32,
}

pub(crate) fn get_vrf_info(
    data: &nlas::InfoData,
) -> Result<Option<VrfInfo>, NisporError> {
    if let nlas::InfoData::Vrf(raw) = data {
        let nla_buff = NlaBuffer::new(raw);
        if nla_buff.kind() == IFLA_VRF_TABLE {
            Ok(Some(VrfInfo {
                table_id: parse_as_u32(nla_buff.value())?,
                subordinates: Vec::new(),
            }))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn get_vrf_subordinate_info(
    data: &[u8],
) -> Result<Option<VrfSubordinateInfo>, NisporError> {
    let nla_buff = NlaBuffer::new(data);
    if nla_buff.kind() == IFLA_VRF_PORT_TABLE {
        Ok(Some(VrfSubordinateInfo {
            table_id: parse_as_u32(nla_buff.value())?,
        }))
    } else {
        Ok(None)
    }
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
                        let mut new_subordinates: Vec<String> = Vec::new();
                        new_subordinates.push(iface.name.clone());
                        controller_subordinates
                            .insert(controller.clone(), new_subordinates);
                    }
                };
            }
        }
    }
    for (controller, subordinates) in controller_subordinates.iter_mut() {
        if let Some(controller_iface) = iface_states.get_mut(controller) {
            if let Some(old_vrf_info) = &controller_iface.vrf {
                // TODO: Need better way to update this subordinate list.
                let mut new_vrf_info = old_vrf_info.clone();
                subordinates.sort();
                new_vrf_info.subordinates = subordinates.clone();
                controller_iface.vrf = Some(new_vrf_info);
            }
        }
    }
}
