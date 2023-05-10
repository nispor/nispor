// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::rtnl::route::nlas::Nla;
use rtnetlink::RouteGetRequest;

use crate::{NisporError, Route, RouteProtocol, RouteScope};

#[derive(Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct NetStateRouteFilter {
    /// Returned routes will only contain routes from specified protocol.
    pub protocol: Option<RouteProtocol>,
    /// Returned routes will only contain routes from specified scope.
    pub scope: Option<RouteScope>,
    /// Returned routes will only contain routes next hop to specified
    /// interface.
    pub oif: Option<String>,
    /// Returned routes will only contain routes in specified route table.
    pub table: Option<u8>,
}

pub(crate) fn apply_kernel_route_filter(
    handle: &mut RouteGetRequest,
    filter: &NetStateRouteFilter,
    iface_name2index: &HashMap<String, u32>,
) -> Result<(), NisporError> {
    let rt_nlmsg = handle.message_mut();

    if let Some(protocol) = filter.protocol.as_ref() {
        rt_nlmsg.header.protocol = protocol.into();
    }
    if let Some(scope) = filter.scope.as_ref() {
        rt_nlmsg.header.scope = scope.into();
    }
    if let Some(oif) = filter.oif.as_ref() {
        match iface_name2index.get(oif) {
            Some(index) => rt_nlmsg.nlas.push(Nla::Oif(*index)),
            None => {
                let e = NisporError::invalid_argument(format!(
                    "Interface {oif} not found"
                ));
                log::error!("{}", e);
                return Err(e);
            }
        }
    }
    if let Some(table) = filter.table {
        rt_nlmsg.nlas.push(Nla::Table(table.into()));
    }
    Ok(())
}

pub(crate) fn should_drop_by_filter(
    route: &Route,
    filter: &NetStateRouteFilter,
    has_kernel_filter: bool,
) -> bool {
    // The RT_SCOPE_UNIVERSE is 0 which means wildcard in kernel, we need to
    // do filter at userspace.
    if Some(&RouteScope::Universe) == filter.scope.as_ref() {
        route.scope != RouteScope::Universe
    } else {
        if !has_kernel_filter
            && ((filter.protocol.is_some()
                && filter.protocol != Some(route.protocol))
                || (filter.scope.is_some()
                    && filter.scope.as_ref() != Some(&route.scope))
                || (filter.oif.is_some()
                    && filter.oif.as_ref() != route.oif.as_ref())
                || (filter.table.is_some()
                    && filter.table.as_ref().map(|i| (*i).into())
                        != Some(route.table)))
        {
            return true;
        }
        false
    }
}
