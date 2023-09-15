// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::net::IpAddr;

use netlink_packet_route::{
    route::nlas::Nla, RouteMessage, RTN_UNICAST, RTPROT_STATIC,
    RT_SCOPE_UNIVERSE, RT_TABLE_MAIN,
};

use serde::{Deserialize, Serialize};

use super::super::netlink::{AF_INET, AF_INET6};
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
    nl_msg.header.kind = RTN_UNICAST;
    if let Some(p) = route.protocol.as_ref() {
        nl_msg.header.protocol = p.into();
    } else {
        nl_msg.header.protocol = RTPROT_STATIC;
    }
    nl_msg.header.scope = RT_SCOPE_UNIVERSE;
    nl_msg.header.table = RT_TABLE_MAIN;
    let (dst_addr, dst_prefix) = parse_ip_net_addr_str(route.dst.as_str())?;
    nl_msg.header.destination_prefix_length = dst_prefix;
    match dst_addr {
        IpAddr::V4(addr) => {
            nl_msg.header.address_family = AF_INET;
            nl_msg.nlas.push(Nla::Destination(addr.octets().to_vec()));
        }
        IpAddr::V6(addr) => {
            nl_msg.header.address_family = AF_INET6;
            nl_msg.nlas.push(Nla::Destination(addr.octets().to_vec()));
        }
    };
    if let Some(t) = route.table.as_ref() {
        nl_msg.header.table = *t;
    }
    if let Some(m) = route.metric.as_ref() {
        nl_msg.nlas.push(Nla::Priority(*m));
    }
    if let Some(oif) = route.oif.as_deref() {
        if let Some(iface_index) = iface_name_2_index.get(oif) {
            nl_msg.nlas.push(Nla::Iif(*iface_index));
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
                nl_msg.nlas.push(Nla::Gateway(i.octets().to_vec()));
            }
            IpAddr::V6(i) => {
                nl_msg.nlas.push(Nla::Gateway(i.octets().to_vec()));
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
        req.message_mut().nlas = nl_msg.nlas;
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
