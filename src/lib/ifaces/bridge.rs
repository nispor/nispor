use crate::netlink::parse_bridge_port_info;
use crate::parse_as_mac;
use crate::Iface;
use crate::MasterType;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::link::nlas::InfoBridge::{
    AgeingTime, BridgeId, ForwardDelay, GroupFwdMask, HelloTime, MaxAge,
    MulticastSnooping, Priority, StpState, VlanDefaultPvid, VlanFiltering,
    VlanProtocol,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::transmute;

const ETH_ALEN: usize = 6;
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
pub struct BridgeStpInfo {
    pub state: BridgeStpState,
    pub hello_time: u32,
    pub forward_delay: u32,
    pub max_age: u32,
    pub priority: u16,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgeVlanFilteringInfo {
    pub enabled: bool,
    pub vlan_protocol: BridgeVlanProtocol,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_pvid: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BridgeInfo {
    pub slaves: Vec<String>,
    pub stp: BridgeStpInfo,
    pub ageing_time: u32,
    pub bridge_id: String,
    pub vlan_filtering: BridgeVlanFilteringInfo,
    pub group_fwd_mask: u16,
    pub multicast_snooping: bool,
    // TODO: There are a lot remaining properties
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
}

pub(crate) fn get_bridge_info(data: &nlas::InfoData) -> Option<BridgeInfo> {
    let mut bridge_info = BridgeInfo::default();
    if let nlas::InfoData::Bridge(infos) = data {
        for info in infos {
            if let StpState(d) = info {
                bridge_info.stp.state = (*d).into();
            } else if let VlanFiltering(d) = info {
                bridge_info.vlan_filtering.enabled = *d > 0;
            } else if let HelloTime(d) = info {
                bridge_info.stp.hello_time = *d;
            } else if let MaxAge(d) = info {
                bridge_info.stp.max_age = *d;
            } else if let ForwardDelay(d) = info {
                bridge_info.stp.forward_delay = *d;
            } else if let Priority(d) = info {
                bridge_info.stp.priority = *d;
            } else if let AgeingTime(d) = info {
                bridge_info.ageing_time = *d;
            } else if let BridgeId((priority, mac)) = info {
                //Following the format of sysfs
                let priority_bytes = priority.to_ne_bytes();
                bridge_info.bridge_id = format!(
                    "{:02x}{:02x}.{}",
                    priority_bytes[0],
                    priority_bytes[1],
                    parse_as_mac(ETH_ALEN, mac).to_lowercase().replace(":", "")
                )
            } else if let VlanProtocol(d) = info {
                //TODO: Once https://github.com/little-dude/netlink/pull/78
                //released, remove the endian converting line
                let protocol = u16::from_be_bytes(d.to_ne_bytes());
                bridge_info.vlan_filtering.vlan_protocol = protocol.into();
            } else if let VlanDefaultPvid(d) = info {
                bridge_info.vlan_filtering.default_pvid = Some((*d).into());
            } else if let GroupFwdMask(d) = info {
                bridge_info.group_fwd_mask = *d;
            } else if let MulticastSnooping(d) = info {
                bridge_info.multicast_snooping = *d > 0;
            } else {
                ()
                // println!("{:?}", &info);
            }
        }
        Some(bridge_info)
    } else {
        None
    }
}

pub(crate) fn get_bridge_port_info(data: &[u8]) -> Option<BridgePortInfo> {
    Some(parse_bridge_port_info(data))
}

pub(crate) fn bridge_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_slave_list_of_master(iface_states);
    convert_back_port_index_to_name(iface_states);
}

// TODO: This is duplicate of bond gen_slave_list_of_master()
fn gen_slave_list_of_master(iface_states: &mut HashMap<String, Iface>) {
    let mut master_slaves: HashMap<String, Vec<String>> = HashMap::new();
    for iface in iface_states.values() {
        if iface.master_type == Some(MasterType::Bridge) {
            if let Some(master) = &iface.master {
                match master_slaves.get_mut(master) {
                    Some(slaves) => slaves.push(iface.name.clone()),
                    None => {
                        let mut new_slaves: Vec<String> = Vec::new();
                        new_slaves.push(iface.name.clone());
                        master_slaves.insert(master.clone(), new_slaves);
                    }
                };
            }
        }
    }
    for (master, slaves) in master_slaves.iter_mut() {
        if let Some(master_iface) = iface_states.get_mut(master) {
            if let Some(old_bridge_info) = &master_iface.bridge_info {
                // TODO: Need better way to update this slave list.
                let mut new_bridge_info = old_bridge_info.clone();
                slaves.sort();
                new_bridge_info.slaves = slaves.clone();
                master_iface.bridge_info = Some(new_bridge_info);
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
        if iface.master_type != Some(MasterType::Bridge) {
            continue;
        }
        if let Some(old_port_info) = &iface.bridge_port_info {
            let index = &old_port_info.backup_port;
            if index != "" {
                if let Some(iface_name) = index_to_name.get(index) {
                    // TODO: Find a way to update old_port_info instaed of
                    // clone()
                    let mut new_port_info = old_port_info.clone();
                    new_port_info.backup_port = iface_name.into();
                    iface.bridge_port_info = Some(new_port_info);
                }
            }
        }
    }
}
