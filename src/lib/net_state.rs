use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::Iface;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
}

pub fn get_state() -> Result<NetState, NisporError> {
    Ok(NetState {
        ifaces: get_ifaces(),
    })
}
