// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    net::{Ipv4Addr, Ipv6Addr},
};

use futures::stream::TryStreamExt;
use rtnetlink::{
    IpVersion, RouteMessageBuilder, new_connection,
    packet_route::route::{
        self as rt, RouteAddress, RouteAttribute, RouteMessage, RouteMetric,
        RouteVia,
    },
    sys::AsyncSocket,
};
use serde::{Deserialize, Serialize};

use super::super::filter::{apply_kernel_route_filter, should_drop_by_filter};
use crate::{ErrorKind, NetStateRouteFilter, NisporError};

const USER_HZ: u32 = 100;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct Route {
    pub address_family: AddressFamily,
    pub tos: u8,
    pub table: u32,
    pub protocol: RouteProtocol,
    pub scope: RouteScope,
    pub route_type: RouteType,
    pub flags: Vec<RouteFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oif: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iif: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefered_src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<RouteRealm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mark: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<u32>,

    // Below are RTAX_* of RTA_METRICS
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mtu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rtt: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rttvar: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssthresh: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwnd: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advmss: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reordering: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hoplimit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initcwnd: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub features: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rto_min: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initrwnd: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quickack: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cc_algo: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fastopen_no_cookie: Option<u32>,

    // Below are RTM_CACHEINFO
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_clntref: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_last_use: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_expires: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_error: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_used: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ts: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_ts_age: Option<u32>,

    // Below are IPv6 only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metric: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preference: Option<RoutePreference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multipath: Option<Vec<MultipathRoute>>,
    // Missing support of RTA_NH_ID
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum AddressFamily {
    #[default]
    Ipv4,
    Ipv6,
    Other(u8),
}

impl From<rtnetlink::packet_route::AddressFamily> for AddressFamily {
    fn from(d: rtnetlink::packet_route::AddressFamily) -> Self {
        match d {
            rtnetlink::packet_route::AddressFamily::Inet => AddressFamily::Ipv4,
            rtnetlink::packet_route::AddressFamily::Inet6 => {
                AddressFamily::Ipv6
            }
            _ => Self::Other(u8::from(d)),
        }
    }
}

impl From<AddressFamily> for rtnetlink::packet_route::AddressFamily {
    fn from(v: AddressFamily) -> Self {
        match v {
            AddressFamily::Ipv4 => rtnetlink::packet_route::AddressFamily::Inet,
            AddressFamily::Ipv6 => {
                rtnetlink::packet_route::AddressFamily::Inet6
            }
            AddressFamily::Other(d) => d.into(),
        }
    }
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RouteProtocol {
    #[default]
    Unspec,
    #[serde(rename = "icmp-redirect")]
    IcmpRedirect,
    Kernel,
    Boot,
    Static,
    Gated,
    Ra,
    #[serde(rename = "merit-mrt")]
    Mrt,
    Zebra,
    Bird,
    #[serde(rename = "decnet-routing-daemon")]
    DnRouted,
    Xorp,
    #[serde(rename = "netsukuku")]
    Ntk,
    Dhcp,
    #[serde(rename = "multicast-daemon")]
    Mrouted,
    #[serde(rename = "keepalived-daemon")]
    KeepAlived,
    Babel,
    Bgp,
    Isis,
    Ospf,
    Rip,
    Eigrp,
    Other(u8),
}

impl From<rt::RouteProtocol> for RouteProtocol {
    fn from(d: rt::RouteProtocol) -> Self {
        match d {
            rt::RouteProtocol::Unspec => RouteProtocol::Unspec,
            rt::RouteProtocol::IcmpRedirect => RouteProtocol::IcmpRedirect,
            rt::RouteProtocol::Kernel => RouteProtocol::Kernel,
            rt::RouteProtocol::Boot => RouteProtocol::Boot,
            rt::RouteProtocol::Static => RouteProtocol::Static,
            rt::RouteProtocol::Gated => RouteProtocol::Gated,
            rt::RouteProtocol::Ra => RouteProtocol::Ra,
            rt::RouteProtocol::Mrt => RouteProtocol::Mrt,
            rt::RouteProtocol::Zebra => RouteProtocol::Zebra,
            rt::RouteProtocol::Bird => RouteProtocol::Bird,
            rt::RouteProtocol::DnRouted => RouteProtocol::DnRouted,
            rt::RouteProtocol::Xorp => RouteProtocol::Xorp,
            rt::RouteProtocol::Ntk => RouteProtocol::Ntk,
            rt::RouteProtocol::Dhcp => RouteProtocol::Dhcp,
            rt::RouteProtocol::Mrouted => RouteProtocol::Mrouted,
            rt::RouteProtocol::KeepAlived => RouteProtocol::KeepAlived,
            rt::RouteProtocol::Babel => RouteProtocol::Babel,
            rt::RouteProtocol::Bgp => RouteProtocol::Bgp,
            rt::RouteProtocol::Isis => RouteProtocol::Isis,
            rt::RouteProtocol::Ospf => RouteProtocol::Ospf,
            rt::RouteProtocol::Rip => RouteProtocol::Rip,
            rt::RouteProtocol::Eigrp => RouteProtocol::Eigrp,
            _ => RouteProtocol::Other(d.into()),
        }
    }
}

impl From<RouteProtocol> for rt::RouteProtocol {
    fn from(v: RouteProtocol) -> Self {
        match v {
            RouteProtocol::Unspec => rt::RouteProtocol::Unspec,
            RouteProtocol::IcmpRedirect => rt::RouteProtocol::IcmpRedirect,
            RouteProtocol::Kernel => rt::RouteProtocol::Kernel,
            RouteProtocol::Boot => rt::RouteProtocol::Boot,
            RouteProtocol::Static => rt::RouteProtocol::Static,
            RouteProtocol::Gated => rt::RouteProtocol::Gated,
            RouteProtocol::Ra => rt::RouteProtocol::Ra,
            RouteProtocol::Mrt => rt::RouteProtocol::Mrt,
            RouteProtocol::Zebra => rt::RouteProtocol::Zebra,
            RouteProtocol::Bird => rt::RouteProtocol::Bird,
            RouteProtocol::DnRouted => rt::RouteProtocol::DnRouted,
            RouteProtocol::Xorp => rt::RouteProtocol::Xorp,
            RouteProtocol::Ntk => rt::RouteProtocol::Ntk,
            RouteProtocol::Dhcp => rt::RouteProtocol::Dhcp,
            RouteProtocol::Mrouted => rt::RouteProtocol::Mrouted,
            RouteProtocol::KeepAlived => rt::RouteProtocol::KeepAlived,
            RouteProtocol::Babel => rt::RouteProtocol::Babel,
            RouteProtocol::Bgp => rt::RouteProtocol::Bgp,
            RouteProtocol::Isis => rt::RouteProtocol::Isis,
            RouteProtocol::Ospf => rt::RouteProtocol::Ospf,
            RouteProtocol::Rip => rt::RouteProtocol::Rip,
            RouteProtocol::Eigrp => rt::RouteProtocol::Eigrp,
            RouteProtocol::Other(d) => d.into(),
        }
    }
}

impl TryFrom<&str> for RouteProtocol {
    type Error = NisporError;

    fn try_from(v: &str) -> Result<Self, NisporError> {
        match v {
            "icmp-redirect" => Ok(RouteProtocol::IcmpRedirect),
            "kernel" => Ok(RouteProtocol::Kernel),
            "boot" => Ok(RouteProtocol::Boot),
            "static" => Ok(RouteProtocol::Static),
            "gated" => Ok(RouteProtocol::Gated),
            "ra" => Ok(RouteProtocol::Ra),
            "merit-mrt" => Ok(RouteProtocol::Mrt),
            "zebra" => Ok(RouteProtocol::Zebra),
            "bird" => Ok(RouteProtocol::Bird),
            "decnet-routing-daemon" => Ok(RouteProtocol::DnRouted),
            "xorp" => Ok(RouteProtocol::Xorp),
            "netsukuku" => Ok(RouteProtocol::Ntk),
            "Dhcp" => Ok(RouteProtocol::Dhcp),
            "multicast-daemon" => Ok(RouteProtocol::Mrouted),
            "keepalived-daemon" => Ok(RouteProtocol::KeepAlived),
            "babel" => Ok(RouteProtocol::Babel),
            "bgp" => Ok(RouteProtocol::Bgp),
            "isis" => Ok(RouteProtocol::Isis),
            "ospf" => Ok(RouteProtocol::Ospf),
            "rip" => Ok(RouteProtocol::Rip),
            "eigrp" => Ok(RouteProtocol::Eigrp),
            _ => Err(NisporError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid route protocol: {v}"),
            )),
        }
    }
}

/*
 * Kernel Doc for route scope:
 * Really it is not scope, but sort of distance to the destination.
 * NOWHERE are reserved for not existing destinations, HOST is our
 * local addresses, LINK are destinations, located on directly attached
 * link and UNIVERSE is everywhere in the Universe.
 * Intermediate values are also possible f.e. interior routes
 * could be assigned a value between UNIVERSE and LINK.
 */
#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RouteScope {
    #[default]
    Universe,
    Site,
    Link,
    Host,
    #[serde(rename = "no-where")]
    NoWhere,
    Other(u8),
}

impl From<rt::RouteScope> for RouteScope {
    fn from(d: rt::RouteScope) -> Self {
        match d {
            rt::RouteScope::Universe => RouteScope::Universe,
            rt::RouteScope::Site => RouteScope::Site,
            rt::RouteScope::Link => RouteScope::Link,
            rt::RouteScope::Host => RouteScope::Host,
            rt::RouteScope::NoWhere => RouteScope::NoWhere,
            _ => RouteScope::Other(d.into()),
        }
    }
}

impl From<RouteScope> for rt::RouteScope {
    fn from(v: RouteScope) -> rt::RouteScope {
        match v {
            RouteScope::Universe => rt::RouteScope::Universe,
            RouteScope::Site => rt::RouteScope::Site,
            RouteScope::Link => rt::RouteScope::Link,
            RouteScope::Host => rt::RouteScope::Host,
            RouteScope::NoWhere => rt::RouteScope::NoWhere,
            RouteScope::Other(d) => d.into(),
        }
    }
}

impl TryFrom<&str> for RouteScope {
    type Error = NisporError;

    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v {
            "u" | "universe" | "g" | "global" => Ok(RouteScope::Universe),
            "s" | "site" => Ok(RouteScope::Site),
            "l" | "link" => Ok(RouteScope::Link),
            "h" | "host" => Ok(RouteScope::Host),
            "n" | "nowhere" | "no_where" => Ok(RouteScope::NoWhere),
            _ => Err(NisporError::new(
                ErrorKind::InvalidArgument,
                format!("Unknown route scope {v}"),
            )),
        }
    }
}

impl std::fmt::Display for RouteScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Universe => write!(f, "universe"),
            Self::Site => write!(f, "site"),
            Self::Link => write!(f, "link"),
            Self::Host => write!(f, "host"),
            Self::NoWhere => write!(f, "no-where"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum RouteType {
    #[default]
    Unspec,
    Unicast,
    Local,
    Broadcast,
    Anycast,
    Multicast,
    BlackHole,
    Unreachable,
    Prohibit,
    Throw,
    Nat,
    ExternalResolve,
    Other(u8),
}

impl From<rt::RouteType> for RouteType {
    fn from(d: rt::RouteType) -> Self {
        match d {
            rt::RouteType::Unspec => RouteType::Unspec,
            rt::RouteType::Unicast => RouteType::Unicast,
            rt::RouteType::Local => RouteType::Local,
            rt::RouteType::Broadcast => RouteType::Broadcast,
            rt::RouteType::Anycast => RouteType::Anycast,
            rt::RouteType::Multicast => RouteType::Multicast,
            rt::RouteType::BlackHole => RouteType::BlackHole,
            rt::RouteType::Unreachable => RouteType::Unreachable,
            rt::RouteType::Prohibit => RouteType::Prohibit,
            rt::RouteType::Throw => RouteType::Throw,
            rt::RouteType::Nat => RouteType::Nat,
            rt::RouteType::ExternalResolve => RouteType::ExternalResolve,
            _ => RouteType::Other(d.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct MultipathRoute {
    pub via: String,
    pub iface: String,
    pub weight: u16, // The kernel is u8, but ip route show it after + 1.
    pub flags: Vec<MultipathRouteFlag>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum MultipathRouteFlag {
    Dead,
    Pervasive,
    Onlink,
    Offload,
    Linkdown,
    Unresolved,
    Trap,
    Other(u8),
}

impl MultipathRouteFlag {
    pub(crate) fn from_netlink(d: rt::RouteNextHopFlags) -> Vec<Self> {
        d.iter()
            .map(|bit| match bit {
                rt::RouteNextHopFlags::Dead => Self::Dead,
                rt::RouteNextHopFlags::Pervasive => Self::Pervasive,
                rt::RouteNextHopFlags::Onlink => Self::Onlink,
                rt::RouteNextHopFlags::Offload => Self::Offload,
                rt::RouteNextHopFlags::Linkdown => Self::Linkdown,
                rt::RouteNextHopFlags::Unresolved => Self::Unresolved,
                rt::RouteNextHopFlags::Trap => Self::Trap,
                _ => Self::Other(bit.bits()),
            })
            .collect()
    }

    pub(crate) fn to_netlink(flags: &[Self]) -> rt::RouteNextHopFlags {
        let mut ret = rt::RouteNextHopFlags::empty();
        for flag in flags {
            ret |= match flag {
                Self::Dead => rt::RouteNextHopFlags::Dead,
                Self::Pervasive => rt::RouteNextHopFlags::Pervasive,
                Self::Onlink => rt::RouteNextHopFlags::Onlink,
                Self::Offload => rt::RouteNextHopFlags::Offload,
                Self::Linkdown => rt::RouteNextHopFlags::Linkdown,
                Self::Unresolved => rt::RouteNextHopFlags::Unresolved,
                Self::Trap => rt::RouteNextHopFlags::Trap,
                Self::Other(d) => rt::RouteNextHopFlags::from_bits_retain(*d),
            }
        }
        ret
    }
}

pub(crate) async fn get_routes(
    iface_name2index: &HashMap<String, u32>,
    filter: Option<&NetStateRouteFilter>,
) -> Result<Vec<Route>, NisporError> {
    let mut routes = Vec::new();
    let mut has_kernel_filter = true;
    let (mut connection, handle, _) = new_connection()?;

    let mut ifindex_to_name = HashMap::new();
    for (name, index) in iface_name2index.iter() {
        ifindex_to_name.insert(format!("{index}"), name.to_string());
    }

    if let Some(filter) = filter
        && !filter.is_empty()
        && let Err(e) = connection
            .socket_mut()
            .socket_mut()
            .set_netlink_get_strict_chk(true)
    {
        log::warn!(
            "Failed to set kernel space route filter: {e}, falling back to \
             user space route filtering which would lead to performance \
             penalty"
        );
        has_kernel_filter = false;
    }

    tokio::spawn(connection);

    for ip_family in [IpVersion::V6, IpVersion::V4] {
        let rt_msg = match ip_family {
            IpVersion::V4 => RouteMessageBuilder::<Ipv4Addr>::new().build(),
            IpVersion::V6 => RouteMessageBuilder::<Ipv6Addr>::new().build(),
        };
        let mut rt_handle = handle.route().get(rt_msg);
        if let Some(filter) = filter
            && has_kernel_filter
        {
            apply_kernel_route_filter(
                &mut rt_handle,
                filter,
                iface_name2index,
            )?;
        }

        let mut links = rt_handle.execute();
        while let Some(rt_msg) = links.try_next().await? {
            let route = get_route(rt_msg, &ifindex_to_name)?;
            // User space filter is required for RT_SCOPE_UNIVERSE and etc
            if let Some(filter) = filter
                && should_drop_by_filter(&route, filter, has_kernel_filter)
            {
                continue;
            }
            routes.push(route);
        }
    }
    Ok(routes)
}

fn get_route(
    route_msg: RouteMessage,
    ifindex_to_name: &HashMap<String, String>,
) -> Result<Route, NisporError> {
    let mut rt = Route::default();
    let header = &route_msg.header;
    rt.address_family = header.address_family.into();
    let src_prefix_len = header.source_prefix_length;
    let dst_prefix_len = header.destination_prefix_length;
    rt.table = header.table.into();
    rt.tos = header.tos;
    rt.protocol = header.protocol.into();
    rt.scope = header.scope.into();
    rt.flags = header.flags.iter().map(RouteFlag::from).collect();
    rt.route_type = header.kind.into();
    let _family = &rt.address_family;
    for nla in &route_msg.attributes {
        match nla {
            RouteAttribute::Destination(d) => {
                rt.dst = Some(format!(
                    "{}/{}",
                    _rt_addr_to_string(d),
                    dst_prefix_len
                ));
            }
            RouteAttribute::Oif(d) => {
                rt.oif = if let Some(iface_name) =
                    ifindex_to_name.get(&format!("{d}"))
                {
                    Some(iface_name.clone())
                } else {
                    Some(format!("{d}"))
                }
            }
            RouteAttribute::PrefSource(d) => {
                rt.prefered_src = Some(_rt_addr_to_string(d));
            }
            RouteAttribute::Table(d) => {
                rt.table = *d;
            }
            RouteAttribute::Realm(d) => {
                rt.realm = Some((*d).into());
            }
            RouteAttribute::Source(d) => {
                rt.src = Some(format!(
                    "{}/{}",
                    _rt_addr_to_string(d),
                    src_prefix_len
                ));
            }
            RouteAttribute::Gateway(d) => {
                rt.gateway = Some(_rt_addr_to_string(d));
            }
            RouteAttribute::Via(d) => {
                if let RouteVia::Inet(a) = d {
                    rt.via = Some(a.to_string());
                } else if let RouteVia::Inet6(a) = d {
                    rt.via = Some(a.to_string());
                }
            }
            RouteAttribute::Metrics(nlas) => {
                for nla in nlas {
                    match nla {
                        RouteMetric::Lock(d) => {
                            rt.lock = Some(*d);
                        }
                        RouteMetric::Mtu(d) => {
                            rt.mtu = Some(*d);
                        }
                        RouteMetric::Window(d) => {
                            rt.window = Some(*d);
                        }
                        RouteMetric::Rtt(d) => {
                            rt.rtt = Some(*d);
                        }
                        RouteMetric::RttVar(d) => {
                            rt.rttvar = Some(*d);
                        }
                        RouteMetric::SsThresh(d) => {
                            rt.ssthresh = Some(*d);
                        }
                        RouteMetric::Cwnd(d) => {
                            rt.cwnd = Some(*d);
                        }
                        RouteMetric::Advmss(d) => {
                            rt.advmss = Some(*d);
                        }
                        RouteMetric::Reordering(d) => {
                            rt.reordering = Some(*d);
                        }
                        RouteMetric::Hoplimit(d) => {
                            rt.hoplimit = Some(*d);
                        }
                        RouteMetric::InitCwnd(d) => {
                            rt.initcwnd = Some(*d);
                        }
                        RouteMetric::Features(d) => {
                            rt.features = Some(*d);
                        }
                        RouteMetric::RtoMin(d) => {
                            rt.rto_min = Some(*d);
                        }
                        RouteMetric::InitRwnd(d) => {
                            rt.initrwnd = Some(*d);
                        }
                        RouteMetric::QuickAck(d) => {
                            rt.quickack = Some(*d);
                        }
                        RouteMetric::CcAlgo(d) => {
                            rt.cc_algo = Some(*d);
                        }
                        RouteMetric::FastopenNoCookie(d) => {
                            rt.fastopen_no_cookie = Some(*d);
                        }
                        _ => {
                            log::debug!("Unknown RTA_METRICS message {nla:?}");
                        }
                    }
                }
            }

            RouteAttribute::Mark(d) => {
                rt.mark = Some(*d);
            }
            RouteAttribute::Uid(d) => {
                rt.uid = Some(*d);
            }
            RouteAttribute::Iif(d) => {
                rt.iif = if let Some(iface_name) =
                    ifindex_to_name.get(&format!("{d}"))
                {
                    Some(iface_name.clone())
                } else {
                    Some(format!("{d}"))
                }
            }
            RouteAttribute::CacheInfo(d) => {
                rt.cache_clntref = Some(d.clntref);
                rt.cache_last_use = Some(d.last_use);
                rt.cache_expires = Some(d.expires / USER_HZ);
                rt.cache_error = Some(d.error);
                rt.cache_used = Some(d.used);
                rt.cache_id = Some(d.id);
                rt.cache_ts = Some(d.ts);
                rt.cache_ts_age = Some(d.ts_age);
            }
            RouteAttribute::Priority(d) => {
                rt.metric = Some(*d);
            }
            RouteAttribute::MultiPath(hops) => {
                let mut next_hops = Vec::new();
                for hop in hops.as_slice() {
                    let mut mp_rt = MultipathRoute::default();
                    for nla in hop.attributes.iter() {
                        if let RouteAttribute::Gateway(v) = nla {
                            if let RouteAddress::Inet(v) = v {
                                mp_rt.via = v.to_string();
                            } else if let RouteAddress::Inet6(v) = v {
                                mp_rt.via = v.to_string();
                            }
                            break;
                        }
                    }
                    let iface_index = hop.interface_index;
                    mp_rt.iface = if let Some(iface_name) =
                        ifindex_to_name.get(&format!("{iface_index}"))
                    {
                        iface_name.clone()
                    } else {
                        format!("{iface_index}")
                    };
                    mp_rt.flags = MultipathRouteFlag::from_netlink(hop.flags);
                    // +1 because ip route does so
                    mp_rt.weight = u16::from(hop.hops) + 1;
                    next_hops.push(mp_rt);
                }
                rt.multipath = Some(next_hops);
            }
            RouteAttribute::Preference(d) => rt.preference = Some((*d).into()),
            _ => log::debug!("Unknown NLA message for route {nla:?}"),
        }
    }

    Ok(rt)
}

fn _rt_addr_to_string(addr: &RouteAddress) -> String {
    match addr {
        RouteAddress::Inet(v) => v.to_string(),
        RouteAddress::Inet6(v) => v.to_string(),
        _ => {
            log::debug!("Unknown RouteAddress type {addr:?}");
            String::new()
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Default, Serialize, Deserialize,
)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RoutePreference {
    Low,
    #[default]
    Medium,
    High,
    Invalid,
    Other(u8),
}

impl From<rt::RoutePreference> for RoutePreference {
    fn from(d: rt::RoutePreference) -> Self {
        match d {
            rt::RoutePreference::Low => Self::Low,
            rt::RoutePreference::Medium => Self::Medium,
            rt::RoutePreference::High => Self::High,
            rt::RoutePreference::Invalid => Self::Invalid,
            _ => Self::Other(d.into()),
        }
    }
}

#[derive(
    Clone, Eq, PartialEq, Debug, Copy, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct RouteRealm {
    pub source: u16,
    pub destination: u16,
}

impl From<rt::RouteRealm> for RouteRealm {
    fn from(d: rt::RouteRealm) -> Self {
        Self {
            source: d.source,
            destination: d.destination,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Copy, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum RouteFlag {
    Dead,
    Pervasive,
    Onlink,
    Offload,
    Linkdown,
    Unresolved,
    Trap,
    Notify,
    Cloned,
    Equalize,
    Prefix,
    LookupTable,
    FibMatch,
    RtOffload,
    RtTrap,
    OffloadFailed,
    Other(u32),
}

impl From<rt::RouteFlags> for RouteFlag {
    fn from(d: rt::RouteFlags) -> Self {
        match d {
            rt::RouteFlags::Dead => Self::Dead,
            rt::RouteFlags::Pervasive => Self::Pervasive,
            rt::RouteFlags::Onlink => Self::Onlink,
            rt::RouteFlags::Offload => Self::Offload,
            rt::RouteFlags::Linkdown => Self::Linkdown,
            rt::RouteFlags::Unresolved => Self::Unresolved,
            rt::RouteFlags::Trap => Self::Trap,
            rt::RouteFlags::Notify => Self::Notify,
            rt::RouteFlags::Cloned => Self::Cloned,
            rt::RouteFlags::Equalize => Self::Equalize,
            rt::RouteFlags::Prefix => Self::Prefix,
            rt::RouteFlags::LookupTable => Self::LookupTable,
            rt::RouteFlags::FibMatch => Self::FibMatch,
            rt::RouteFlags::RtOffload => Self::RtOffload,
            rt::RouteFlags::RtTrap => Self::RtTrap,
            rt::RouteFlags::OffloadFailed => Self::OffloadFailed,
            _ => Self::Other(d.bits()),
        }
    }
}
