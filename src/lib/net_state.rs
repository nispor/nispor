use crate::error::ZatelError;
use crate::iface_state::get_ifaces;
use crate::iface_state::Iface;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
}

pub fn get_state() -> Result<NetState, ZatelError> {
    Ok(NetState {
        ifaces: get_ifaces(),
    })
}
