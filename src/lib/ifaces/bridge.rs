use crate::netlink::parse_af_spec_bridge_info;
use crate::netlink::parse_bridge_info;
use crate::netlink::parse_bridge_port_info;
use crate::Iface;
use crate::ControllerType;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::transmute;

const ETH_P_8021Q: u16 = 0x8100;
const ETH_P_8021AD: u16 = 0x88A8;

#[repr(u32)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BridgeStpState {
    Disabled,
    KernelStp,
    UserStp,
    Unknown,
}

impl Default for BridgeStpState {
    fn default() -> Self {
        BridgeStpState::Unknown
    }
}

const _LAST_STP_TYPE: BridgeStpState = BridgeStpState::UserStp;

impl From<u32> for BridgeStpState {
    fn from(d: u32) -> Self {
        if d <= _LAST_STP_TYPE as u32 {
            unsafe { transmute(d as u32) }
        } else {
            BridgeStpState::Unknown
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BridgeVlanProtocol {
    #[serde(rename = "802.1Q")]
    Ieee8021Q,
    #[serde(rename = "802.1AD")]
    Ieee8021AD,
    Unknown,
}

impl Default for BridgeVlanProtocol {
    fn default() -> Self {
        BridgeVlanProtocol::Unknown
    }
}

impl From<u16> for BridgeVlanProtocol {
    fn from(d: u16) -> Self {
        match d {
            ETH_P_8021Q => BridgeVlanProtocol::Ieee8021Q,
            ETH_P_8021AD => BridgeVlanProtocol::Ieee8021AD,
            _ => BridgeVlanProtocol::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgeVlanFilteringInfo {}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgeInfo {
    pub ports: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ageing_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_fwd_mask: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_path_cost: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_change: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_change_detected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcn_timer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topology_change_timer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gc_timer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_addr: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_iptables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_ip6tables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_arptables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_filtering: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_protocol: Option<BridgeVlanProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_pvid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_per_host: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_state: Option<BridgeStpState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hello_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hello_timer: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_delay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multi_bool_opt: Option<u64>, // does not avaiable in sysfs yet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_router: Option<BridgePortMulticastRouterType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_snooping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_query_use_ifaddr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_querier: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_stats_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_hash_elasticity: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_hash_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_last_member_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_last_member_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_startup_query_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_membership_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_querier_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_query_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_query_response_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_startup_query_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_igmp_version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_mld_version: Option<u8>,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BridgePortStpState {
    Disabled,
    Listening,
    Learning,
    Forwarding,
    Blocking,
    Unknown,
}

const _LAST_PORT_STP_STATE: BridgePortStpState = BridgePortStpState::Blocking;

impl Default for BridgePortStpState {
    fn default() -> Self {
        BridgePortStpState::Unknown
    }
}

impl From<u8> for BridgePortStpState {
    fn from(d: u8) -> Self {
        if d <= _LAST_PORT_STP_STATE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BridgePortStpState::Unknown
        }
    }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BridgePortMulticastRouterType {
    Disabled,
    TempQuery,
    Perm,
    Temp,
    Unknown,
}

const _LAST_PORT_MDB_RTR_TYPE: BridgePortMulticastRouterType =
    BridgePortMulticastRouterType::Temp;

impl Default for BridgePortMulticastRouterType {
    fn default() -> Self {
        BridgePortMulticastRouterType::Unknown
    }
}

impl From<u8> for BridgePortMulticastRouterType {
    fn from(d: u8) -> Self {
        if d <= _LAST_PORT_MDB_RTR_TYPE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BridgePortMulticastRouterType::Unknown
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgePortInfo {
    pub stp_state: BridgePortStpState,
    pub stp_priority: u16,
    pub stp_path_cost: u32,
    pub hairpin_mode: bool,
    pub bpdu_guard: bool,
    pub root_block: bool,
    pub multicast_fast_leave: bool,
    pub learning: bool,
    pub unicast_flood: bool,
    pub proxyarp: bool,
    pub proxyarp_wifi: bool,
    pub designated_root: String,
    pub designated_bridge: String,
    pub designated_port: u16,
    pub designated_cost: u16,
    pub port_id: String,
    pub port_no: String,
    pub change_ack: bool,
    pub config_pending: bool,
    pub message_age_timer: u64,
    pub forward_delay_timer: u64,
    pub hold_timer: u64,
    pub multicast_router: BridgePortMulticastRouterType,
    pub multicast_flood: bool,
    pub multicast_to_unicast: bool,
    pub vlan_tunnel: bool,
    pub broadcast_flood: bool,
    pub group_fwd_mask: u16,
    pub neigh_suppress: bool,
    pub isolated: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub backup_port: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlans: Option<Vec<BridgeVlanEntry>>,
}

pub(crate) fn get_bridge_info(data: &nlas::InfoData) -> Option<BridgeInfo> {
    if let nlas::InfoData::Bridge(infos) = data {
        Some(parse_bridge_info(&infos))
    } else {
        None
    }
}

pub(crate) fn get_bridge_port_info(data: &[u8]) -> Option<BridgePortInfo> {
    Some(parse_bridge_port_info(data))
}

pub(crate) fn bridge_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_port_list_of_controller(iface_states);
    convert_back_port_index_to_name(iface_states);
}

// TODO: This is duplicate of bond gen_port_list_of_controller()
fn gen_port_list_of_controller(iface_states: &mut HashMap<String, Iface>) {
    let mut controller_ports: HashMap<String, Vec<String>> = HashMap::new();
    for iface in iface_states.values() {
        if iface.controller_type == Some(ControllerType::Bridge) {
            if let Some(controller) = &iface.controller {
                match controller_ports.get_mut(controller) {
                    Some(ports) => ports.push(iface.name.clone()),
                    None => {
                        let mut new_ports: Vec<String> = Vec::new();
                        new_ports.push(iface.name.clone());
                        controller_ports.insert(controller.clone(), new_ports);
                    }
                };
            }
        }
    }
    for (controller, ports) in controller_ports.iter_mut() {
        if let Some(controller_iface) = iface_states.get_mut(controller) {
            if let Some(old_bridge_info) = &controller_iface.bridge {
                // TODO: Need better way to update this port list.
                let mut new_bridge_info = old_bridge_info.clone();
                ports.sort();
                new_bridge_info.ports = ports.clone();
                controller_iface.bridge = Some(new_bridge_info);
            }
        }
    }
}

fn convert_back_port_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.controller_type != Some(ControllerType::Bridge) {
            continue;
        }
        if let Some(old_port_info) = &iface.bridge_port {
            let index = &old_port_info.backup_port;
            if index != "" {
                if let Some(iface_name) = index_to_name.get(index) {
                    // TODO: Find a way to update old_port_info instaed of
                    // clone()
                    let mut new_port_info = old_port_info.clone();
                    new_port_info.backup_port = iface_name.into();
                    iface.bridge_port = Some(new_port_info);
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgeVlanEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid_range: Option<(u16, u16)>,
    pub is_pvid: bool, // is PVID and ingress untagged
    pub is_egress_untagged: bool,
}

pub(crate) fn parse_bridge_vlan_info(iface_state: &mut Iface, data: &[u8]) {
    if let Some(old_port_info) = &iface_state.bridge_port {
        // TODO: shoule update in place instead of clone
        let mut new_port_info = old_port_info.clone();
        new_port_info.vlans = parse_af_spec_bridge_info(data);
        iface_state.bridge_port = Some(new_port_info);
    }
}
