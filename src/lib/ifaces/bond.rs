use crate::ifaces::Iface;
use crate::netlink::parse_bond_info;
use crate::netlink::parse_bond_slave_info;
use crate::IfaceType;
use crate::MasterType;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::transmute;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondInfo {
    pub slaves: Vec<String>,
    pub mode: String,
    pub options: HashMap<String, String>,
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondSlaveState {
    Active,
    Backup,
    Unknown = std::u8::MAX,
}

const _LAST_BOND_SLAVE_STATE: BondSlaveState = BondSlaveState::Backup;

impl From<u8> for BondSlaveState {
    fn from(d: u8) -> Self {
        if d <= _LAST_BOND_SLAVE_STATE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondSlaveState::Unknown
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
pub struct BondSlaveInfo {
    pub slave_state: BondSlaveState,
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
            slaves: Vec::new(), // TODO
            mode: mode,
            options: bond_info,
        })
    } else {
        None
    }
}

pub(crate) fn get_bond_slave_info(data: &[u8]) -> Option<BondSlaveInfo> {
    Some(parse_bond_slave_info(data))
}

pub(crate) fn bond_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_slave_list_of_master(iface_states);
    active_slave_index_to_iface_name(iface_states);
}

fn gen_slave_list_of_master(iface_states: &mut HashMap<String, Iface>) {
    let mut master_slaves: HashMap<String, Vec<String>> = HashMap::new();
    for iface in iface_states.values() {
        if iface.master_type == Some(MasterType::Bond) {
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
            if let Some(old_bond_info) = &master_iface.bond {
                // TODO: Need better way to update this slave list.
                let mut new_bond_info = old_bond_info.clone();
                slaves.sort();
                new_bond_info.slaves = slaves.clone();
                master_iface.bond = Some(new_bond_info);
            }
        }
    }
}

fn active_slave_index_to_iface_name(iface_states: &mut HashMap<String, Iface>) {
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
            if let Some(index) = bond_options.get("active_slave") {
                if let Some(iface_name) = index_to_name.get(index) {
                    bond_options
                        .insert("active_slave".into(), iface_name.into());
                }
            }
            let mut bond_info = old_bond_info.clone();
            bond_info.options = bond_options;
            iface.bond = Some(bond_info);
        }
    }
}
