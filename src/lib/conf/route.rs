// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::net::IpAddr;

use netlink_packet_route::{
    route::{self as rt, RouteAddress, RouteAttribute, RouteMessage},
    AddressFamily,
};

use serde::{Deserialize, Serialize};

use super::super::query::{parse_ip_addr_str, parse_ip_net_addr_str};
use crate::{NisporError, RouteProtocol};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct RouteConf {
    #[serde(default)]
    pub remove: bool,
    pub dst: String,
    pub oif: Option<String>,
    pub via: Option<String>,
    pub metric: Option<u32>,
    pub table: Option<u8>,
    pub protocol: Option<RouteProtocol>,
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
    let mut nl_msg = RouteMessage::default();
    nl_msg.header.kind = rt::RouteType::Unicast;
    if let Some(p) = route.protocol {
        nl_msg.header.protocol = p.into();
    } else {
        nl_msg.header.protocol = rt::RouteProtocol::Static;
    }
    nl_msg.header.scope = rt::RouteScope::Universe;
    nl_msg.header.table = rt::RouteHeader::RT_TABLE_MAIN;
    let (dst_addr, dst_prefix) = parse_ip_net_addr_str(route.dst.as_str())?;
    nl_msg.header.destination_prefix_length = dst_prefix;
    match dst_addr {
        IpAddr::V4(addr) => {
            nl_msg.header.address_family = AddressFamily::Inet;
            nl_msg
                .attributes
                .push(RouteAttribute::Destination(RouteAddress::Inet(addr)));
        }
        IpAddr::V6(addr) => {
            nl_msg.header.address_family = AddressFamily::Inet6;
            nl_msg
                .attributes
                .push(RouteAttribute::Destination(RouteAddress::Inet6(addr)));
        }
    };
    if let Some(t) = route.table.as_ref() {
        nl_msg.header.table = *t;
    }
    if let Some(m) = route.metric.as_ref() {
        nl_msg.attributes.push(RouteAttribute::Priority(*m));
    }
    if let Some(oif) = route.oif.as_deref() {
        if let Some(iface_index) = iface_name_2_index.get(oif) {
            nl_msg.attributes.push(RouteAttribute::Iif(*iface_index));
        } else {
            let e = NisporError::invalid_argument(format!(
                "Interface {oif} does not exist"
            ));
            log::error!("{}", e);
            return Err(e);
        }
    }
    if let Some(via) = route.via.as_deref() {
        match parse_ip_addr_str(via)? {
            IpAddr::V4(i) => {
                nl_msg
                    .attributes
                    .push(RouteAttribute::Gateway(RouteAddress::Inet(i)));
            }
            IpAddr::V6(i) => {
                nl_msg
                    .attributes
                    .push(RouteAttribute::Gateway(RouteAddress::Inet6(i)));
            }
        };
    }
    if route.remove {
        if let Err(e) = handle.route().del(nl_msg).execute().await {
            if let rtnetlink::Error::NetlinkError(ref e) = e {
                if e.raw_code() == -libc::ESRCH {
                    return Ok(());
                }
            }
            return Err(e.into());
        }
    } else {
        let mut req = handle.route().add();
        req.message_mut().header = nl_msg.header;
        req.message_mut().attributes = nl_msg.attributes;
        if let Err(e) = req.execute().await {
            if let rtnetlink::Error::NetlinkError(ref e) = e {
                if e.raw_code() == -libc::EEXIST {
                    return Ok(());
                }
            }
            return Err(e.into());
        }
    }
    Ok(())
}
