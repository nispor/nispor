use crate::ifaces::Iface;
use crate::netlink::parse_bond_info;
use crate::netlink::parse_bond_subordinate_info;
use crate::ControllerType;
use crate::IfaceType;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::transmute;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondInfo {
    pub subordinates: Vec<String>,
    pub mode: String,
    pub options: HashMap<String, String>,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondSubordinateState {
    Active,
    Backup,
    Unknown = std::u8::MAX,
}

const _LAST_BOND_SUBORDINATE_STATE: BondSubordinateState =
    BondSubordinateState::Backup;

impl From<u8> for BondSubordinateState {
    fn from(d: u8) -> Self {
        if d <= _LAST_BOND_SUBORDINATE_STATE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondSubordinateState::Unknown
        }
    }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondMiiStatus {
    LinkUp,
    LinkFail,
    LinkDown,
    LinkBack,
    Unknown = std::u8::MAX,
}

const _LAST_MII_STATUS: BondMiiStatus = BondMiiStatus::LinkBack;

impl From<u8> for BondMiiStatus {
    fn from(d: u8) -> Self {
        if d <= _LAST_MII_STATUS as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondMiiStatus::Unknown
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondSubordinateInfo {
    pub subordinate_state: BondSubordinateState,
    pub mii_status: BondMiiStatus,
    pub link_failure_count: u32,
    pub perm_hwaddr: String,
    pub queue_id: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_aggregator_id: Option<u16>,
    // 802.3ad port state definitions (43.4.2.2 in the 802.3ad standard)
    // bit map of LACP_STATE_XXX
    // TODO: Find a rust way of showing it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_actor_oper_port_state: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_partner_oper_port_state: Option<u16>,
}

pub(crate) fn get_bond_info(data: &nlas::InfoData) -> Option<BondInfo> {
    if let nlas::InfoData::Bond(raw) = data {
        let mut bond_info = parse_bond_info(raw);
        let mode = match bond_info.get("mode") {
            Some(m) => m.to_string(),
            None => "unknown".into(),
        };
        bond_info.remove("mode");
        Some(BondInfo {
            subordinates: Vec::new(), // TODO
            mode: mode,
            options: bond_info,
        })
    } else {
        None
    }
}

pub(crate) fn get_bond_subordinate_info(
    data: &[u8],
) -> Option<BondSubordinateInfo> {
    Some(parse_bond_subordinate_info(data))
}

pub(crate) fn bond_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_subordinate_list_of_controller(iface_states);
    active_subordinate_index_to_iface_name(iface_states);
}

fn gen_subordinate_list_of_controller(
    iface_states: &mut HashMap<String, Iface>,
) {
    let mut controller_subordinates: HashMap<String, Vec<String>> =
        HashMap::new();
    for iface in iface_states.values() {
        if iface.controller_type == Some(ControllerType::Bond) {
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
            if let Some(old_bond_info) = &controller_iface.bond {
                // TODO: Need better way to update this subordinate list.
                let mut new_bond_info = old_bond_info.clone();
                subordinates.sort();
                new_bond_info.subordinates = subordinates.clone();
                controller_iface.bond = Some(new_bond_info);
            }
        }
    }
}

fn active_subordinate_index_to_iface_name(
    iface_states: &mut HashMap<String, Iface>,
) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Bond {
            continue;
        }
        if let Some(old_bond_info) = &iface.bond {
            let mut bond_options = old_bond_info.options.clone();
            if let Some(index) = bond_options.get("active_subordinate") {
                if let Some(iface_name) = index_to_name.get(index) {
                    bond_options
                        .insert("active_subordinate".into(), iface_name.into());
                }
            }
            let mut bond_info = old_bond_info.clone();
            bond_info.options = bond_options;
            iface.bond = Some(bond_info);
        }
    }
}
