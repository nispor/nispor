// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_route::link::{
    self, InfoBridge, InfoBridgePort, InfoData,
};
use serde::{Deserialize, Serialize};

use crate::{
    mac::{parse_as_mac, ETH_ALEN},
    BridgeVlanEntry, ControllerType, Iface, NisporError, VlanProtocol,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BridgeStpState {
    Disabled,
    KernelStp,
    UserStp,
    Other(u32),
}

impl From<link::BridgeStpState> for BridgeStpState {
    fn from(d: link::BridgeStpState) -> Self {
        match d {
            link::BridgeStpState::Disabled => Self::Disabled,
            link::BridgeStpState::KernelStp => Self::KernelStp,
            link::BridgeStpState::UserStp => Self::UserStp,
            _ => Self::Other(d.into()),
        }
    }
}

impl From<BridgeStpState> for link::BridgeStpState {
    fn from(d: BridgeStpState) -> Self {
        match d {
            BridgeStpState::Disabled => Self::Disabled,
            BridgeStpState::KernelStp => Self::KernelStp,
            BridgeStpState::UserStp => Self::UserStp,
            BridgeStpState::Other(d) => Self::Other(d.into()),
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
    pub vlan_protocol: Option<VlanProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_pvid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_per_port: Option<bool>,
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
    pub multicast_router: Option<BridgeMulticastRouterType>,
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
#[derive(Default)]
pub enum BridgePortStpState {
    Disabled,
    Listening,
    Learning,
    Forwarding,
    Blocking,
    Other(u8),
    #[default]
    Unknown,
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
#[derive(Default)]
pub enum BridgeMulticastRouterType {
    #[default]
    Disabled,
    TempQuery,
    Perm,
    Temp,
    Other(u8),
}

impl From<link::BridgeMulticastRouterType> for BridgeMulticastRouterType {
    fn from(d: link::BridgeMulticastRouterType) -> Self {
        match d {
            link::BridgeMulticastRouterType::Disabled => Self::Disabled,
            link::BridgeMulticastRouterType::TempQuery => Self::TempQuery,
            link::BridgeMulticastRouterType::Permanent => Self::Perm,
            link::BridgeMulticastRouterType::Temp => Self::Temp,
            _ => Self::Other(d.into()),
        }
    }
}

impl From<BridgeMulticastRouterType> for link::BridgeMulticastRouterType {
    fn from(value: BridgeMulticastRouterType) -> Self {
        match value {
            BridgeMulticastRouterType::Disabled => {
                link::BridgeMulticastRouterType::Disabled
            }
            BridgeMulticastRouterType::TempQuery => {
                link::BridgeMulticastRouterType::TempQuery
            }
            BridgeMulticastRouterType::Perm => {
                link::BridgeMulticastRouterType::Permanent
            }
            BridgeMulticastRouterType::Temp => {
                link::BridgeMulticastRouterType::Temp
            }
            BridgeMulticastRouterType::Other(d) => {
                link::BridgeMulticastRouterType::Other(d)
            }
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
    pub multicast_router: BridgeMulticastRouterType,
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
                ret.multicast_router = (*d).into()
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
                log::info!("Unknown bridge port info {nla:?}");
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

fn parse_bridge_info(infos: &[InfoBridge]) -> Result<BridgeInfo, NisporError> {
    let mut bridge_info = BridgeInfo::default();

    for info in infos {
        if let InfoBridge::ForwardDelay(d) = info {
            bridge_info.forward_delay = Some(*d);
        } else if let InfoBridge::HelloTime(d) = info {
            bridge_info.hello_time = Some(*d);
        } else if let InfoBridge::MaxAge(d) = info {
            bridge_info.max_age = Some(*d);
        } else if let InfoBridge::AgeingTime(d) = info {
            bridge_info.ageing_time = Some(*d);
        } else if let InfoBridge::StpState(d) = info {
            bridge_info.stp_state = Some((*d).into());
        } else if let InfoBridge::Priority(d) = info {
            bridge_info.priority = Some(*d);
        } else if let InfoBridge::VlanFiltering(d) = info {
            bridge_info.vlan_filtering = Some(*d);
        } else if let InfoBridge::VlanProtocol(d) = info {
            bridge_info.vlan_protocol = Some((*d).into());
        } else if let InfoBridge::GroupFwdMask(d) = info {
            bridge_info.group_fwd_mask = Some(*d);
        } else if let InfoBridge::RootId(bridge_id) = info {
            bridge_info.root_id = Some(parse_bridge_id(bridge_id)?);
        } else if let InfoBridge::BridgeId(bridge_id) = info {
            bridge_info.bridge_id = Some(parse_bridge_id(bridge_id)?);
        } else if let InfoBridge::RootPort(d) = info {
            bridge_info.root_port = Some(*d);
        } else if let InfoBridge::RootPathCost(d) = info {
            bridge_info.root_path_cost = Some(*d);
        } else if let InfoBridge::TopologyChange(d) = info {
            bridge_info.topology_change = Some(*d > 0);
        } else if let InfoBridge::TopologyChangeDetected(d) = info {
            bridge_info.topology_change_detected = Some(*d > 0);
        } else if let InfoBridge::HelloTimer(d) = info {
            bridge_info.hello_timer = Some(*d);
        } else if let InfoBridge::TcnTimer(d) = info {
            bridge_info.tcn_timer = Some(*d);
        } else if let InfoBridge::TopologyChangeTimer(d) = info {
            bridge_info.topology_change_timer = Some(*d);
        } else if let InfoBridge::GcTimer(d) = info {
            bridge_info.gc_timer = Some(*d);
        } else if let InfoBridge::GroupAddr(d) = info {
            bridge_info.group_addr = Some(parse_as_mac(ETH_ALEN, d)?);
        // InfoBridge::FdbFlush is only used for changing bridge
        } else if let InfoBridge::MulticastRouter(d) = info {
            bridge_info.multicast_router = Some((*d).into());
        } else if let InfoBridge::MulticastSnooping(d) = info {
            bridge_info.multicast_snooping = Some(*d);
        } else if let InfoBridge::MulticastQueryUseIfaddr(d) = info {
            bridge_info.multicast_query_use_ifaddr = Some(*d);
        } else if let InfoBridge::MulticastQuerier(d) = info {
            bridge_info.multicast_querier = Some(*d);
        } else if let InfoBridge::MulticastHashElasticity(d) = info {
            bridge_info.multicast_hash_elasticity = Some(*d);
        } else if let InfoBridge::MulticastHashMax(d) = info {
            bridge_info.multicast_hash_max = Some(*d);
        } else if let InfoBridge::MulticastLastMemberCount(d) = info {
            bridge_info.multicast_last_member_count = Some(*d);
        } else if let InfoBridge::MulticastStartupQueryCount(d) = info {
            bridge_info.multicast_startup_query_count = Some(*d);
        } else if let InfoBridge::MulticastLastMemberInterval(d) = info {
            bridge_info.multicast_last_member_interval = Some(*d);
        } else if let InfoBridge::MulticastMembershipInterval(d) = info {
            bridge_info.multicast_membership_interval = Some(*d);
        } else if let InfoBridge::MulticastQuerierInterval(d) = info {
            bridge_info.multicast_querier_interval = Some(*d);
        } else if let InfoBridge::MulticastQueryInterval(d) = info {
            bridge_info.multicast_query_interval = Some(*d);
        } else if let InfoBridge::MulticastQueryResponseInterval(d) = info {
            bridge_info.multicast_query_response_interval = Some(*d);
        } else if let InfoBridge::MulticastStartupQueryInterval(d) = info {
            bridge_info.multicast_startup_query_interval = Some(*d);
        } else if let InfoBridge::NfCallIpTables(d) = info {
            bridge_info.nf_call_iptables = Some(*d);
        } else if let InfoBridge::NfCallIp6Tables(d) = info {
            bridge_info.nf_call_ip6tables = Some(*d);
        } else if let InfoBridge::NfCallArpTables(d) = info {
            bridge_info.nf_call_arptables = Some(*d);
        } else if let InfoBridge::VlanDefaultPvid(d) = info {
            bridge_info.default_pvid = Some(*d);
        } else if let InfoBridge::VlanStatsEnabled(d) = info {
            bridge_info.vlan_stats_enabled = Some(*d);
        } else if let InfoBridge::MulticastStatsEnabled(d) = info {
            bridge_info.multicast_stats_enabled = Some(*d);
        } else if let InfoBridge::MulticastIgmpVersion(d) = info {
            bridge_info.multicast_igmp_version = Some(*d);
        } else if let InfoBridge::MulticastMldVersion(d) = info {
            bridge_info.multicast_mld_version = Some(*d);
        } else if let InfoBridge::VlanStatsPerPort(d) = info {
            bridge_info.vlan_stats_per_port = Some(*d);
        } else {
            log::debug!("Unknown NLA {:?}", &info);
        }
    }
    Ok(bridge_info)
}

fn parse_bridge_id(bridge_id: &link::BridgeId) -> Result<String, NisporError> {
    let mac = parse_as_mac(ETH_ALEN, &bridge_id.address)
        .map_err(|_| {
            NisporError::invalid_argument(
                "invalid mac address in bridge_id".into(),
            )
        })?
        .to_lowercase()
        .replace(':', "");

    Ok(format!("{:04x}.{}", bridge_id.priority, mac))
}
