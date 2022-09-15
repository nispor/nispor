// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::runtime;

use crate::{
    error::NisporError,
    ifaces::{get_iface_name2index, get_ifaces, Iface},
    mptcp::{get_mptcp, merge_mptcp_info, Mptcp},
    route::{get_routes, Route},
    route_rule::{get_route_rules, RouteRule},
    NetStateRouteFilter,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct NetState {
    pub ifaces: HashMap<String, Iface>,
    pub routes: Vec<Route>,
    pub rules: Vec<RouteRule>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mptcp: Option<Mptcp>,
}

impl NetState {
    pub fn retrieve() -> Result<NetState, NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        let mut ifaces = rt.block_on(get_ifaces())?;
        let mut ifname_to_index = HashMap::new();
        for iface in ifaces.values() {
            ifname_to_index.insert(iface.name.clone(), iface.index);
        }
        let routes = rt.block_on(get_routes(&ifname_to_index, None))?;

        let rules = rt.block_on(get_route_rules())?;
        let mut mptcp = rt.block_on(get_mptcp())?;
        merge_mptcp_info(&mut ifaces, &mut mptcp);
        Ok(NetState {
            ifaces,
            routes,
            rules,
            mptcp: Some(mptcp),
        })
    }

    // TODO: autoconvert NetState to NetConf and provide apply() here
}

pub fn retrieve_routes_with_filter(
    filter: &NetStateRouteFilter,
) -> Result<Vec<Route>, NisporError> {
    let rt = runtime::Builder::new_current_thread().enable_io().build()?;
    let iface_name2index = rt.block_on(get_iface_name2index())?;
    rt.block_on(get_routes(&iface_name2index, Some(filter)))
}
