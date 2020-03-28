use crate::error::ZatelError;
use crate::iface_state::get_ifaces;
use crate::iface_state::IfaceState;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub iface_states: HashMap<String, IfaceState>,
}

pub fn get_state() -> Result<NetState, ZatelError> {
    Ok(NetState {
        iface_states: get_ifaces(),
    })
}
