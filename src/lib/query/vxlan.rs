// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{InfoData, InfoVxlan};
use serde::{Deserialize, Serialize};

use crate::{
    netlink::{parse_as_ipv4, parse_as_ipv6},
    Iface, IfaceType, NisporError,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
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
    if let InfoData::Vxlan(infos) = data {
        let mut vxlan_info = VxlanInfo::default();
        for info in infos {
            if let InfoVxlan::Id(d) = *info {
                vxlan_info.vxlan_id = d;
            } else if let InfoVxlan::Group(d) = info {
                vxlan_info.remote = parse_as_ipv4(d)?.to_string();
            } else if let InfoVxlan::Group6(d) = info {
                vxlan_info.remote = parse_as_ipv6(d)?.to_string();
            } else if let InfoVxlan::Link(d) = *info {
                vxlan_info.base_iface = format!("{d}");
            } else if let InfoVxlan::Local(d) = info {
                vxlan_info.local = parse_as_ipv4(d)?.to_string();
            } else if let InfoVxlan::Local6(d) = info {
                vxlan_info.local = parse_as_ipv6(d)?.to_string();
            } else if let InfoVxlan::Tos(d) = *info {
                vxlan_info.tos = d;
            } else if let InfoVxlan::Ttl(d) = *info {
                vxlan_info.ttl = d;
            } else if let InfoVxlan::Learning(d) = *info {
                vxlan_info.learning = d;
            } else if let InfoVxlan::Label(d) = *info {
                vxlan_info.label = d;
            } else if let InfoVxlan::Ageing(d) = *info {
                vxlan_info.ageing = d;
            } else if let InfoVxlan::Limit(d) = *info {
                vxlan_info.max_address = d;
            } else if let InfoVxlan::PortRange(d) = *info {
                vxlan_info.src_port_min = d.0;
                vxlan_info.src_port_max = d.1;
            } else if let InfoVxlan::Proxy(d) = *info {
                vxlan_info.proxy = d;
            } else if let InfoVxlan::Rsc(d) = *info {
                vxlan_info.rsc = d;
            } else if let InfoVxlan::L2Miss(d) = *info {
                vxlan_info.l2miss = d;
            } else if let InfoVxlan::L3Miss(d) = *info {
                vxlan_info.l3miss = d;
            } else if let InfoVxlan::Port(d) = *info {
                vxlan_info.dst_port = d;
            } else if let InfoVxlan::UDPCsum(d) = *info {
                vxlan_info.udp_check_sum = d;
            } else if let InfoVxlan::UDPZeroCsumTX(d) = *info {
                vxlan_info.udp6_zero_check_sum_tx = d;
            } else if let InfoVxlan::UDPZeroCsumRX(d) = *info {
                vxlan_info.udp6_zero_check_sum_rx = d;
            } else if let InfoVxlan::RemCsumTX(d) = *info {
                vxlan_info.remote_check_sum_tx = d;
            } else if let InfoVxlan::RemCsumRX(d) = *info {
                vxlan_info.remote_check_sum_rx = d;
            } else if let InfoVxlan::Gpe(d) = *info {
                vxlan_info.gpe = d;
            } else if let InfoVxlan::Gbp(d) = *info {
                vxlan_info.gbp = d;
            } else if let InfoVxlan::TtlInherit(d) = *info {
                vxlan_info.ttl_inherit = d;
            } else if let InfoVxlan::CollectMetadata(d) = *info {
                vxlan_info.collect_metadata = d;
            } else if let InfoVxlan::Df(d) = *info {
                vxlan_info.df = d;
            } else {
                log::debug!("Unknown VXLAN info {:?}", info)
            }
        }
        Ok(Some(vxlan_info))
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
        if let Some(ref mut vxlan_info) = iface.vxlan {
            if let Some(base_iface_name) =
                index_to_name.get(&vxlan_info.base_iface)
            {
                vxlan_info.base_iface = base_iface_name.clone();
            }
        }
    }
}
