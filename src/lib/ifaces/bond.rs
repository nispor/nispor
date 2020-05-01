use crate::ifaces::netlink::parse_bond_info;
use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondConf {
    pub slaves: Vec<String>,
    pub mode: String,
    pub options: HashMap<String, String>,
}

pub(crate) fn get_bond_conf(data: &nlas::InfoData) -> Option<BondConf> {
    if let nlas::InfoData::Bond(raw) = data {
        let mut bond_info = parse_bond_info(raw);
        let mode = match bond_info.get("mode") {
            Some(m) => m.to_string(),
            None => "unknown".into(),
        };
        bond_info.remove("mode");
        Some(BondConf {
            slaves: Vec::new(), // TODO
            mode: mode,
            options: bond_info,
        })
    } else {
        None
    }
}
