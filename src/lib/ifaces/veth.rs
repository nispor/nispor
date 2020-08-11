use crate::Iface;
use crate::IfaceType;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct VethInfo {
    // Interface name of peer.
    // Use interface index number when peer interface is in other namespace.
    pub peer: String,
}

pub(crate) fn veth_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }

    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Veth {
            continue;
        }

        if let Some(VethInfo { peer }) = &iface.veth {
            if let Some(peer_iface_name) = index_to_name.get(peer) {
                iface.veth = Some(VethInfo {
                    peer: peer_iface_name.clone(),
                })
            }
        }
    }
}
