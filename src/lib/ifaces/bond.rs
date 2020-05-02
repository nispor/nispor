use crate::ifaces::netlink::parse_bond_info;
use crate::ifaces::netlink::parse_bond_slave_info;
use crate::ifaces::netlink::BondSlaveInfo;
use crate::Iface;
use crate::IfaceType;
use crate::MasterType;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondInfo {
    pub slaves: Vec<String>,
    pub mode: String,
    pub options: HashMap<String, String>,
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
    gathering_slaves(iface_states);
    active_slave_index_to_iface_name(iface_states);
}

fn gathering_slaves(iface_states: &mut HashMap<String, Iface>) {
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
            if let Some(old_bond_info) = &master_iface.bond_info {
                // TODO: Need better way to update this slave list.
                let mut new_bond_info = old_bond_info.clone();
                slaves.sort();
                new_bond_info.slaves = slaves.clone();
                master_iface.bond_info = Some(new_bond_info);
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
        if let Some(old_bond_info) = &iface.bond_info {
            let mut bond_options = old_bond_info.options.clone();
            if let Some(index) = bond_options.get("active_slave") {
                if let Some(iface_name) = index_to_name.get(index) {
                    bond_options
                        .insert("active_slave".into(), iface_name.into());
                }
            }
            let mut bond_info = old_bond_info.clone();
            bond_info.options = bond_options;
            iface.bond_info = Some(bond_info);
        }
    }
}
