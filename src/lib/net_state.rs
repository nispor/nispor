// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tokio::runtime;

use super::query::{
    get_ifaces, get_mptcp, get_route_rules, get_routes, merge_mptcp_info,
};
use crate::{
    Iface, Mptcp, NetStateFilter, NetStateIfaceFilter, NisporError, Route,
    RouteRule,
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
        Self::retrieve_with_filter(&NetStateFilter::default())
    }

    // TODO: autoconvert NetState to NetConf and provide apply() here

    pub fn retrieve_with_filter(
        filter: &NetStateFilter,
    ) -> Result<NetState, NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        rt.block_on(Self::retrieve_with_filter_async(filter))
    }

    pub async fn retrieve_with_filter_async(
        filter: &NetStateFilter,
    ) -> Result<NetState, NisporError> {
        let mut ifaces = if filter.iface.is_none() {
            get_ifaces(Some(&NetStateIfaceFilter::minimum())).await?
        } else {
            get_ifaces(filter.iface.as_ref()).await?
        };

        let mut ifname_to_index = HashMap::new();
        for iface in ifaces.values() {
            ifname_to_index.insert(iface.name.clone(), iface.index);
        }

        let routes = if filter.route.is_some() {
            get_routes(&ifname_to_index, filter.route.as_ref()).await?
        } else {
            Vec::new()
        };

        let rules = if filter.route_rule.is_some() {
            get_route_rules().await?
        } else {
            Vec::new()
        };

        let mptcp =
            if filter.iface.as_ref().map(|f| f.include_mptcp) == Some(true) {
                let mut mptcp = get_mptcp().await?;
                merge_mptcp_info(&mut ifaces, &mut mptcp);
                Some(mptcp)
            } else {
                None
            };
        if filter.iface.is_none() {
            ifaces = HashMap::new();
        }
        Ok(NetState {
            ifaces,
            routes,
            rules,
            mptcp,
        })
    }
}
