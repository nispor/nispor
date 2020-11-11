use crate::netlink::parse_vxlan_info;
use crate::Iface;
use crate::IfaceType;
use crate::NisporError;
use netlink_packet_route::rtnl::link::nlas::InfoData;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct VxlanInfo {
    pub remote: String,
    pub vxlan_id: u32,
    pub base_iface: String,
    pub local: String,
    pub ttl: u8,
    pub tos: u8,
    pub learning: bool,
    pub ageing: u32,
    pub max_address: u32,
    pub src_port_min: u16,
    pub src_port_max: u16,
    pub proxy: bool,
    pub rsc: bool,
    pub l2miss: bool,
    pub l3miss: bool,
    pub dst_port: u16,
    pub udp_check_sum: bool,
    pub udp6_zero_check_sum_tx: bool,
    pub udp6_zero_check_sum_rx: bool,
    pub remote_check_sum_tx: bool,
    pub remote_check_sum_rx: bool,
    pub gbp: bool,
    pub remote_check_sum_no_partial: bool,
    pub collect_metadata: bool,
    pub label: u32,
    pub gpe: bool,
    pub ttl_inherit: bool,
    pub df: u8,
}

pub(crate) fn get_vxlan_info(
    data: &InfoData,
) -> Result<Option<VxlanInfo>, NisporError> {
    if let InfoData::Vxlan(raw) = data {
        Ok(Some(parse_vxlan_info(&raw)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn vxlan_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Vxlan {
            continue;
        }
        if let Some(old_vxlan_info) = &iface.vxlan {
            if let Some(base_iface_name) =
                index_to_name.get(&old_vxlan_info.base_iface)
            {
                let mut new_vxlan_info = old_vxlan_info.clone();
                new_vxlan_info.base_iface = base_iface_name.clone();
                iface.vxlan = Some(new_vxlan_info);
            }
        }
    }
}
