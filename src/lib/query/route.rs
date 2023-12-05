// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use std::os::unix::io::AsRawFd;

use futures::stream::TryStreamExt;
use netlink_packet_route::route::{
    self as rt, RouteAddress, RouteAttribute, RouteMessage, RouteMetric,
    RouteVia,
};

use rtnetlink::{new_connection, IpVersion};
use serde::{Deserialize, Serialize};

use super::super::filter::{
    apply_kernel_route_filter, enable_kernel_strict_check,
    should_drop_by_filter,
};
use crate::{NetStateRouteFilter, NisporError};

const USER_HZ: u32 = 100;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
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
#[serde(rename_all = "lowercase")]
pub enum AddressFamily {
    IPv4,
    IPv6,
    Other(u8),
    #[default]
    Unknown,
}

impl From<netlink_packet_route::AddressFamily> for AddressFamily {
    fn from(d: netlink_packet_route::AddressFamily) -> Self {
        match d {
            netlink_packet_route::AddressFamily::Inet => AddressFamily::IPv4,
            netlink_packet_route::AddressFamily::Inet6 => AddressFamily::IPv6,
            _ => Self::Other(u8::from(d)),
        }
    }
}

impl From<AddressFamily> for netlink_packet_route::AddressFamily {
    fn from(v: AddressFamily) -> Self {
        match v {
            AddressFamily::IPv4 => netlink_packet_route::AddressFamily::Inet,
            AddressFamily::IPv6 => netlink_packet_route::AddressFamily::Inet6,
            AddressFamily::Other(d) => d.into(),
            AddressFamily::Unknown => {
                netlink_packet_route::AddressFamily::Unspec
            }
        }
    }
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum RouteProtocol {
    #[default]
    Unspec,
    #[serde(rename = "icmp_redirect")]
    IcmpRedirect,
    Kernel,
    Boot,
    Static,
    Gated,
    Ra,
    #[serde(rename = "merit_mrt")]
    Mrt,
    Zebra,
    Bird,
    #[serde(rename = "decnet_routing_daemon")]
    DnRouted,
    Xorp,
    #[serde(rename = "netsukuku")]
    Ntk,
    Dhcp,
    #[serde(rename = "multicast_daemon")]
    Mrouted,
    #[serde(rename = "keepalived_daemon")]
    KeepAlived,
    Babel,
    Bgp,
    Isis,
    Ospf,
    Rip,
    Eigrp,
    Unknown,
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
            RouteProtocol::Unknown => rt::RouteProtocol::Unspec,
            RouteProtocol::Other(d) => d.into(),
        }
    }
}

impl From<&str> for RouteProtocol {
    fn from(v: &str) -> Self {
        match v {
            "icmp_redirect" => RouteProtocol::IcmpRedirect,
            "kernel" => RouteProtocol::Kernel,
            "boot" => RouteProtocol::Boot,
            "static" => RouteProtocol::Static,
            "gated" => RouteProtocol::Gated,
            "ra" => RouteProtocol::Ra,
            "merit_mrt" => RouteProtocol::Mrt,
            "zebra" => RouteProtocol::Zebra,
            "bird" => RouteProtocol::Bird,
            "decnet_routing_daemon" => RouteProtocol::DnRouted,
            "xorp" => RouteProtocol::Xorp,
            "netsukuku" => RouteProtocol::Ntk,
            "Dhcp" => RouteProtocol::Dhcp,
            "multicast_daemon" => RouteProtocol::Mrouted,
            "keepalived_daemon" => RouteProtocol::KeepAlived,
            "babel" => RouteProtocol::Babel,
            "bgp" => RouteProtocol::Bgp,
            "isis" => RouteProtocol::Isis,
            "ospf" => RouteProtocol::Ospf,
            "rip" => RouteProtocol::Rip,
            "eigrp" => RouteProtocol::Eigrp,
            _ => RouteProtocol::Unknown,
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
#[serde(rename_all = "lowercase")]
pub enum RouteScope {
    Universe,
    Site,
    Link,
    Host,
    #[serde(rename = "no_where")]
    NoWhere,
    #[default]
    Unknown,
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
            RouteScope::Unknown => rt::RouteScope::Universe,
            RouteScope::Other(d) => d.into(),
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
            Self::NoWhere => write!(f, "no_where"),
            Self::Unknown => write!(f, "unknown"),
            Self::Other(s) => write!(f, "{s}"),
        }
    }
}

impl From<&str> for RouteScope {
    fn from(v: &str) -> Self {
        match v {
            "u" | "universe" | "g" | "global" => RouteScope::Universe,
            "s" | "site" => RouteScope::Site,
            "l" | "link" => RouteScope::Link,
            "h" | "host" => RouteScope::Host,
            "n" | "nowhere" | "no_where" => RouteScope::NoWhere,
            _ => RouteScope::Unknown,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "lowercase")]
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
    Unknown,
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
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub struct MultipathRoute {
    pub via: String,
    pub iface: String,
    pub weight: u16, // The kernel is u8, but ip route show it after + 1.
    pub flags: Vec<MultipathRouteFlags>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum MultipathRouteFlags {
    Dead,
    Pervasive,
    #[serde(rename = "on_link")]
    OnLink,
    Offload,
    #[serde(rename = "link_down")]
    LinkDown,
    Unresolved,
    Trap,
    Other(u8),
}

impl From<rt::RouteNextHopFlag> for MultipathRouteFlags {
    fn from(d: rt::RouteNextHopFlag) -> Self {
        match d {
            rt::RouteNextHopFlag::Dead => Self::Dead,
            rt::RouteNextHopFlag::Pervasive => Self::Pervasive,
            rt::RouteNextHopFlag::Onlink => Self::OnLink,
            rt::RouteNextHopFlag::Offload => Self::Offload,
            rt::RouteNextHopFlag::Linkdown => Self::LinkDown,
            rt::RouteNextHopFlag::Unresolved => Self::Unresolved,
            rt::RouteNextHopFlag::Trap => Self::Trap,
            _ => Self::Other(u8::from(d)),
        }
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

    if filter.is_some() {
        if let Err(e) =
            enable_kernel_strict_check(connection.socket_mut().as_raw_fd())
        {
            log::warn!(
                "Failed to set kernel space route filter: {e}, \
                falling back to user space route filtering which would \
                lead to performance penalty"
            );
            has_kernel_filter = false;
        }
    }

    tokio::spawn(connection);

    for ip_family in [IpVersion::V6, IpVersion::V4] {
        let mut rt_handle = handle.route().get(ip_family);
        if let Some(filter) = filter {
            if has_kernel_filter {
                apply_kernel_route_filter(
                    &mut rt_handle,
                    filter,
                    iface_name2index,
                )?;
            }
        }

        let mut links = rt_handle.execute();
        while let Some(rt_msg) = links.try_next().await? {
            let route = get_route(rt_msg, &ifindex_to_name)?;
            // User space filter is required for RT_SCOPE_UNIVERSE and etc
            if let Some(filter) = filter {
                if should_drop_by_filter(&route, filter, has_kernel_filter) {
                    continue;
                }
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
    rt.flags = header
        .flags
        .as_slice()
        .iter()
        .map(|f| RouteFlag::from(*f))
        .collect();
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
                            log::debug!(
                                "Unknown RTA_METRICS message {:?}",
                                nla
                            );
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
                    mp_rt.flags = hop
                        .flags
                        .as_slice()
                        .iter()
                        .map(|f| MultipathRouteFlags::from(*f))
                        .collect();
                    // +1 because ip route does so
                    mp_rt.weight = u16::from(hop.hops) + 1;
                    next_hops.push(mp_rt);
                }
                rt.multipath = Some(next_hops);
            }
            RouteAttribute::Preference(d) => rt.preference = Some((*d).into()),
            _ => log::debug!("Unknown NLA message for route {:?}", nla),
        }
    }

    Ok(rt)
}

fn _rt_addr_to_string(addr: &RouteAddress) -> String {
    match addr {
        RouteAddress::Inet(v) => v.to_string(),
        RouteAddress::Inet6(v) => v.to_string(),
        _ => {
            log::debug!("Unknown RouteAddress type {:?}", addr);
            String::new()
        }
    }
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Default, Serialize, Deserialize,
)]
#[non_exhaustive]
#[serde(rename_all = "lowercase")]
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
#[serde(rename_all = "lowercase")]
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
#[serde(rename_all = "snake_case")]
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

impl From<rt::RouteFlag> for RouteFlag {
    fn from(d: rt::RouteFlag) -> Self {
        match d {
            rt::RouteFlag::Dead => Self::Dead,
            rt::RouteFlag::Pervasive => Self::Pervasive,
            rt::RouteFlag::Onlink => Self::Onlink,
            rt::RouteFlag::Offload => Self::Offload,
            rt::RouteFlag::Linkdown => Self::Linkdown,
            rt::RouteFlag::Unresolved => Self::Unresolved,
            rt::RouteFlag::Trap => Self::Trap,
            rt::RouteFlag::Notify => Self::Notify,
            rt::RouteFlag::Cloned => Self::Cloned,
            rt::RouteFlag::Equalize => Self::Equalize,
            rt::RouteFlag::Prefix => Self::Prefix,
            rt::RouteFlag::LookupTable => Self::LookupTable,
            rt::RouteFlag::FibMatch => Self::FibMatch,
            rt::RouteFlag::RtOffload => Self::RtOffload,
            rt::RouteFlag::RtTrap => Self::RtTrap,
            rt::RouteFlag::OffloadFailed => Self::OffloadFailed,
            _ => Self::Other(d.into()),
        }
    }
}
