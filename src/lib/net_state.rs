use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::Iface;
use crate::route::get_routes;
use crate::route::Route;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
    pub routes: Vec<Route>,
}

pub fn get_state() -> Result<NetState, NisporError> {
    Ok(NetState {
        ifaces: get_ifaces()?,
        routes: get_routes()?,
    })
}
