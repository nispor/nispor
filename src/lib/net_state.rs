use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::Iface;
use crate::route::get_routes;
use crate::route::Route;
use crate::route_rule::get_route_rules;
use crate::route_rule::RouteRule;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
    pub routes: Vec<Route>,
    pub rules: Vec<RouteRule>,
}

impl NetState {
    pub fn retrieve() -> Result<NetState, NisporError> {
        let ifaces = get_ifaces()?;
        let routes = get_routes(&ifaces)?;
        let rules = get_route_rules()?;
        Ok(NetState {
            ifaces,
            routes,
            rules,
        })
    }
}
