use netlink_packet_route::rtnl::link::nlas;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondMode {
    RoundRobin,
    ActiveBackup,
    BalanceXOR,
    Broadcast,
    IEEE8023AD,
    AdaptiveTransmitLoadBalancing,
    AdaptiveLoadBalancing,
    Unknown,
}

impl Default for BondMode {
    fn default() -> Self {
        return BondMode::Unknown;
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct BondConf {
    pub slaves: Vec<String>,
    pub mode: BondMode,
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
