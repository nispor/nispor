// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, net::IpAddr};

use rtnetlink::{
    RouteMessageBuilder, RouteNextHopBuilder,
    packet_route::route::{self as rt},
};
use serde::{Deserialize, Serialize};

use super::super::query::{parse_ip_addr_str, parse_ip_net_addr_str};
use crate::{ErrorKind, MultipathRouteFlag, NisporError, RouteProtocol};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RouteConf {
    #[serde(default)]
    pub remove: bool,
    pub dst: String,
    pub oif: Option<String>,
    pub via: Option<String>,
    pub metric: Option<u32>,
    pub table: Option<u8>,
    pub protocol: Option<RouteProtocol>,
    /// ECMP(Equal-Cost Multipath Protocol) routes
    pub multipath: Option<Vec<RouteMultipathConf>>,
    /// Pretend the nexthop is directly attached to this link, even if it
    /// does not match any interface prefix
    #[serde(default)]
    pub onlink: bool,
}

pub(crate) async fn apply_routes_conf(
    routes: &[RouteConf],
    iface_name_2_index: &HashMap<String, u32>,
) -> Result<(), NisporError> {
    let (connection, handle, _) = rtnetlink::new_connection()?;
    tokio::spawn(connection);
    for route in routes {
        apply_route_conf(&handle, route, iface_name_2_index).await?;
    }
    Ok(())
}

async fn apply_route_conf(
    handle: &rtnetlink::Handle,
    route: &RouteConf,
    iface_name_2_index: &HashMap<String, u32>,
) -> Result<(), NisporError> {
    let (dst_addr, dst_prefix) = parse_ip_net_addr_str(route.dst.as_str())?;
    let is_ipv6 = dst_addr.is_ipv6();
    let mut builder = RouteMessageBuilder::<IpAddr>::new()
        .destination_prefix(dst_addr, dst_prefix)
        .map_err(|e| {
            NisporError::new(
                ErrorKind::Bug,
                format!(
                    "builder.destination_prefix() failed on {dst_addr}, \
                     {dst_prefix}): {e}"
                ),
            )
        })?
        .scope(rt::RouteScope::Universe)
        .table_id(route.table.unwrap_or(rt::RouteHeader::RT_TABLE_MAIN).into());

    if let Some(p) = route.protocol {
        builder = builder.protocol(p.into());
    } else {
        builder = builder.protocol(rt::RouteProtocol::Static);
    }

    if let Some(m) = route.metric.as_ref() {
        builder = builder.priority(*m);
    }
    if let Some(oif) = route.oif.as_deref() {
        if let Some(iface_index) = iface_name_2_index.get(oif) {
            builder = builder.output_interface(*iface_index);
        } else {
            let e = NisporError::invalid_argument(format!(
                "Interface {oif} does not exist"
            ));
            log::error!("{e}");
            return Err(e);
        }
    }
    if let Some(via) = route.via.as_deref() {
        let ip = parse_ip_addr_str(via)?;
        builder = builder.gateway(ip).map_err(|e| {
            NisporError::new(
                ErrorKind::Bug,
                format!("builder.gateway() failed on {ip}: {e}"),
            )
        })?;
    }

    if route.onlink {
        builder = builder.onlink();
    }

    if let Some(mpaths) = route.multipath.as_ref() {
        let mut hops: Vec<rt::RouteNextHop> = Vec::new();
        for mpath in mpaths {
            let mut np_builder = if is_ipv6 {
                RouteNextHopBuilder::new_ipv6()
            } else {
                RouteNextHopBuilder::new_ipv4()
            };
            if let Some(via) = mpath.via.as_ref() {
                let ip = parse_ip_addr_str(via)?;
                np_builder = np_builder.via(ip).map_err(|e| {
                    NisporError::new(
                        ErrorKind::Bug,
                        format!("next_hop_builder.via() failed on {ip}: {e}"),
                    )
                })?;
            }
            if let Some(w) = mpath.weight.as_ref() {
                np_builder = np_builder.weight((*w - 1) as u8);
            }
            if let Some(iface) = mpath.iface.as_ref() {
                if let Some(iface_index) = iface_name_2_index.get(iface) {
                    np_builder = np_builder.interface(*iface_index);
                } else {
                    let e = NisporError::invalid_argument(format!(
                        "Next hope interface {iface} does not exist"
                    ));
                    log::error!("{e}");
                    return Err(e);
                }
            }
            if !mpath.flags.is_empty() {
                np_builder = np_builder.flags(MultipathRouteFlag::to_netlink(
                    mpath.flags.as_slice(),
                ));
            }
            hops.push(np_builder.build());
        }
        if !hops.is_empty() {
            builder = builder.multipath(hops);
        }
    }

    if route.remove {
        if let Err(e) = handle.route().del(builder.build()).execute().await {
            if let rtnetlink::Error::NetlinkError(ref e) = e
                && e.raw_code() == -libc::ESRCH
            {
                return Ok(());
            }
            return Err(e.into());
        }
    } else if let Err(e) = handle.route().add(builder.build()).execute().await {
        if let rtnetlink::Error::NetlinkError(ref e) = e
            && e.raw_code() == -libc::EEXIST
        {
            return Ok(());
        }
        return Err(e.into());
    }
    Ok(())
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RouteMultipathConf {
    /// nexthop address
    pub via: Option<String>,
    /// weight on route path been selected, in the range of 1 - 256
    pub weight: Option<u16>,
    /// Output interface
    pub iface: Option<String>,
    /// Pretend the nexthop is directly attached to this link
    #[serde(default)]
    pub flags: Vec<MultipathRouteFlag>,
}
