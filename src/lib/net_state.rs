use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::Iface;
use crate::route::get_routes;
use crate::route::Route;
use crate::route_rule::get_route_rules;
use crate::route_rule::RouteRule;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
    pub routes: Vec<Route>,
    pub rules: Vec<RouteRule>,
}

impl NetState {
    pub fn retrieve() -> Result<NetState, NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        let ifaces = rt.block_on(get_ifaces())?;
        let routes = rt.block_on(get_routes(&ifaces))?;
        let rules = rt.block_on(get_route_rules())?;
        Ok(NetState {
            ifaces,
            routes,
            rules,
        })
    }

    // TODO: autoconvert NetState to NetConf and provide apply() here
}
