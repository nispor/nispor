use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BondConf {
    pub slaves: Vec<String>,
    pub options: HashMap<String, String>,
}

pub(crate) fn get_bond_conf(data: &nlas::InfoData) -> Option<BondConf> {
    if let nlas::InfoData::Bond(raw) = data {
        println!("{:?}", raw);
        Some(Default::default())
    } else {
        None
    }
}
