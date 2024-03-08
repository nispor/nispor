// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{AfSpecBridge, InfoBridgePort, InfoData};

use serde::{Deserialize, Serialize};

use crate::{
    netlink::{parse_af_spec_bridge_info, parse_bridge_id, parse_bridge_info},
    ControllerType, Iface, IfaceType, NisporError,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BridgeStpState {
    Disabled,
    KernelStp,
    UserStp,
    Other(u32),
    Unknown,
}

const BR_NO_STP: u32 = 0;
const BR_KERNEL_STP: u32 = 1;
const BR_USER_STP: u32 = 2;

impl From<u32> for BridgeStpState {
    fn from(d: u32) -> Self {
        match d {
            BR_NO_STP => Self::Disabled,
            BR_KERNEL_STP => Self::KernelStp,
            BR_USER_STP => Self::UserStp,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BridgeVlanProtocol {
    #[serde(rename = "802.1q")]
    Ieee8021Q,
    #[serde(rename = "802.1ad")]
    Ieee8021AD,
    Other(u16),
    Unknown,
}

const ETH_P_8021Q: u16 = 0x8100;
const ETH_P_8021AD: u16 = 0x88A8;

impl From<u16> for BridgeVlanProtocol {
    fn from(d: u16) -> Self {
        match d {
            ETH_P_8021Q => Self::Ieee8021Q,
            ETH_P_8021AD => Self::Ieee8021AD,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BridgePortStpState {
    Disabled,
    Listening,
    Learning,
    Forwarding,
    Blocking,
    Other(u8),
    Unknown,
}

impl Default for BridgePortStpState {
    fn default() -> Self {
        Self::Unknown
    }
}

const BR_STATE_DISABLED: u8 = 0;
const BR_STATE_LISTENING: u8 = 1;
const BR_STATE_LEARNING: u8 = 2;
const BR_STATE_FORWARDING: u8 = 3;
const BR_STATE_BLOCKING: u8 = 4;

impl From<u8> for BridgePortStpState {
    fn from(d: u8) -> Self {
        match d {
            BR_STATE_DISABLED => Self::Disabled,
            BR_STATE_LISTENING => Self::Listening,
            BR_STATE_LEARNING => Self::Learning,
            BR_STATE_FORWARDING => Self::Forwarding,
            BR_STATE_BLOCKING => Self::Blocking,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BridgePortMulticastRouterType {
    Disabled,
    TempQuery,
    Perm,
    Temp,
    Other(u8),
    Unknown,
}

impl Default for BridgePortMulticastRouterType {
    fn default() -> Self {
        Self::Unknown
    }
}

const MDB_RTR_TYPE_DISABLED: u8 = 0;
const MDB_RTR_TYPE_TEMP_QUERY: u8 = 1;
const MDB_RTR_TYPE_PERM: u8 = 2;
const MDB_RTR_TYPE_TEMP: u8 = 3;

impl From<u8> for BridgePortMulticastRouterType {
    fn from(d: u8) -> Self {
        match d {
            MDB_RTR_TYPE_DISABLED => Self::Disabled,
            MDB_RTR_TYPE_TEMP_QUERY => Self::TempQuery,
            MDB_RTR_TYPE_PERM => Self::Perm,
            MDB_RTR_TYPE_TEMP => Self::Temp,
            _ => Self::Other(d),
        }
    }
}

impl From<BridgePortMulticastRouterType> for u8 {
    fn from(value: BridgePortMulticastRouterType) -> u8 {
        match value {
            BridgePortMulticastRouterType::Disabled => MDB_RTR_TYPE_DISABLED,
            BridgePortMulticastRouterType::TempQuery => MDB_RTR_TYPE_TEMP_QUERY,
            BridgePortMulticastRouterType::Perm => MDB_RTR_TYPE_PERM,
            BridgePortMulticastRouterType::Temp => MDB_RTR_TYPE_TEMP,
            BridgePortMulticastRouterType::Other(d) => d,
            BridgePortMulticastRouterType::Unknown => u8::MAX,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
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
    pub mrp_ring_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mrp_in_open: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_eht_hosts_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_eht_hosts_cnt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlans: Option<Vec<BridgeVlanEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_authentication_bypass: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_n_groups: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multicast_max_groups: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neigh_vlan_supress: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_nexthop_id: Option<u32>,
}

pub(crate) fn get_bridge_info(
    data: &InfoData,
) -> Result<Option<BridgeInfo>, NisporError> {
    if let InfoData::Bridge(infos) = data {
        Ok(Some(parse_bridge_info(infos)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn get_bridge_port_info(
    nlas: &[InfoBridgePort],
) -> Result<BridgePortInfo, NisporError> {
    let mut ret = BridgePortInfo::default();

    for nla in nlas {
        match nla {
            InfoBridgePort::State(d) => ret.stp_state = u8::from(*d).into(),
            InfoBridgePort::Priority(d) => ret.stp_priority = *d,
            InfoBridgePort::Cost(d) => ret.stp_path_cost = *d,
            InfoBridgePort::HairpinMode(d) => ret.hairpin_mode = *d,
            InfoBridgePort::Guard(d) => ret.bpdu_guard = *d,
            InfoBridgePort::Protect(d) => ret.root_block = *d,
            InfoBridgePort::FastLeave(d) => ret.multicast_fast_leave = *d,
            InfoBridgePort::Learning(d) => ret.learning = *d,
            InfoBridgePort::UnicastFlood(d) => ret.unicast_flood = *d,
            InfoBridgePort::ProxyARP(d) => ret.proxyarp = *d,
            InfoBridgePort::ProxyARPWifi(d) => ret.proxyarp_wifi = *d,
            InfoBridgePort::RootId(d) => {
                ret.designated_root = parse_bridge_id(d)?
            }
            InfoBridgePort::BridgeId(d) => {
                ret.designated_bridge = parse_bridge_id(d)?
            }
            InfoBridgePort::DesignatedPort(d) => ret.designated_port = *d,
            InfoBridgePort::DesignatedCost(d) => ret.designated_cost = *d,
            InfoBridgePort::PortId(d) => ret.port_id = format!("0x{:04x}", *d),
            InfoBridgePort::PortNumber(d) => {
                ret.port_no = format!("0x{:x}", *d)
            }
            InfoBridgePort::TopologyChangeAck(d) => ret.change_ack = *d,
            InfoBridgePort::ConfigPending(d) => ret.config_pending = *d,
            InfoBridgePort::MessageAgeTimer(d) => ret.message_age_timer = *d,
            InfoBridgePort::ForwardDelayTimer(d) => {
                ret.forward_delay_timer = *d
            }
            InfoBridgePort::HoldTimer(d) => ret.hold_timer = *d,
            InfoBridgePort::Flush => (),
            InfoBridgePort::MulticastRouter(d) => {
                ret.multicast_router = u8::from(*d).into()
            }
            InfoBridgePort::MulticastFlood(d) => ret.multicast_flood = *d,
            InfoBridgePort::MulticastToUnicast(d) => {
                ret.multicast_to_unicast = *d
            }
            InfoBridgePort::VlanTunnel(d) => ret.vlan_tunnel = *d,
            InfoBridgePort::BroadcastFlood(d) => ret.broadcast_flood = *d,
            InfoBridgePort::GroupFwdMask(d) => ret.group_fwd_mask = *d,
            InfoBridgePort::NeighSupress(d) => ret.neigh_suppress = *d,
            InfoBridgePort::Isolated(d) => ret.isolated = *d,
            InfoBridgePort::BackupPort(d) => ret.backup_port = d.to_string(),
            InfoBridgePort::MrpRingOpen(d) => ret.mrp_ring_open = Some(*d),
            InfoBridgePort::MrpInOpen(d) => ret.mrp_in_open = Some(*d),
            InfoBridgePort::MulticastEhtHostsLimit(d) => {
                ret.mcast_eht_hosts_limit = Some(*d)
            }
            InfoBridgePort::MulticastEhtHostsCnt(d) => {
                ret.mcast_eht_hosts_cnt = Some(*d)
            }
            InfoBridgePort::Locked(d) => ret.locked = Some(*d),
            InfoBridgePort::Mab(d) => ret.mac_authentication_bypass = Some(*d),
            InfoBridgePort::MulticastNGroups(d) => {
                ret.multicast_n_groups = Some(*d)
            }
            InfoBridgePort::MulticastMaxGroups(d) => {
                ret.multicast_max_groups = Some(*d)
            }
            InfoBridgePort::NeighVlanSupress(d) => {
                ret.neigh_vlan_supress = Some(*d)
            }
            InfoBridgePort::BackupNextHopId(d) => {
                ret.backup_nexthop_id = Some(*d)
            }
            _ => {
                log::info!("Unknown bridge port info {:?}", nla);
            }
        }
    }

    Ok(ret)
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
                        let new_ports: Vec<String> = vec![iface.name.clone()];
                        controller_ports.insert(controller.clone(), new_ports);
                    }
                };
            }
        }
    }
    for (controller, ports) in controller_ports.iter_mut() {
        if let Some(controller_iface) = iface_states.get_mut(controller) {
            if let Some(ref mut bridge_info) = controller_iface.bridge {
                ports.sort();
                bridge_info.ports.clone_from(ports);
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
        if let Some(ref mut port_info) = iface.bridge_port {
            let index = &port_info.backup_port;
            if !index.is_empty() {
                if let Some(iface_name) = index_to_name.get(index) {
                    port_info.backup_port = iface_name.into();
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BridgeVlanEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid_range: Option<(u16, u16)>,
    pub is_pvid: bool, // is PVID and ingress untagged
    pub is_egress_untagged: bool,
}

pub(crate) fn parse_bridge_vlan_info(
    iface_state: &mut Iface,
    nlas: &[AfSpecBridge],
) -> Result<(), NisporError> {
    if let Some(ref mut port_info) = iface_state.bridge_port {
        if let Some(cur_vlans) = parse_af_spec_bridge_info(nlas)? {
            match port_info.vlans.as_mut() {
                Some(vlans) => vlans.extend(cur_vlans),
                None => port_info.vlans = Some(cur_vlans),
            };
        }
    } else if iface_state.iface_type == IfaceType::Bridge {
        let br_vlan = iface_state.bridge_vlan.get_or_insert(Vec::new());
        // It's the VLAN of the bridge itself
        if let Some(cur_vlans) = parse_af_spec_bridge_info(nlas)? {
            br_vlan.extend(cur_vlans);
        }
    }
    Ok(())
}
