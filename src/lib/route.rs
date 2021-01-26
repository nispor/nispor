use crate::ifaces::Iface;
use crate::netlink::parse_as_i32;
use crate::netlink::parse_as_ipv4;
use crate::netlink::parse_as_ipv6;
use crate::netlink::parse_as_u16;
use crate::netlink::parse_as_u32;
use crate::netlink::AF_INET;
use crate::netlink::AF_INET6;
use crate::NisporError;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::nlas::route::CacheInfo;
use netlink_packet_route::rtnl::nlas::route::CacheInfoBuffer;
use netlink_packet_route::rtnl::nlas::route::Metrics;
use netlink_packet_route::rtnl::nlas::route::Nla;
use netlink_packet_route::rtnl::nlas::NlaBuffer;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use netlink_packet_route::rtnl::{
    RTA_GATEWAY, RTA_VIA, RTN_ANYCAST, RTN_BLACKHOLE, RTN_BROADCAST, RTN_LOCAL,
    RTN_MULTICAST, RTN_NAT, RTN_PROHIBIT, RTN_THROW, RTN_UNICAST,
    RTN_UNREACHABLE, RTN_UNSPEC, RTN_XRESOLVE, RT_SCOPE_HOST, RT_SCOPE_LINK,
    RT_SCOPE_NOWHERE, RT_SCOPE_SITE, RT_SCOPE_UNIVERSE,
};
use netlink_packet_route::RouteMessage;
use netlink_packet_utils::traits::Parseable;
use rtnetlink::new_connection;
use rtnetlink::IpVersion;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

const USER_HZ: u32 = 100;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Route {
    pub address_family: AddressFamily,
    pub tos: u8,
    pub table: u32,
    pub protocol: RouteProtocol,
    pub scope: RouteScope,
    pub route_type: RouteType,
    pub flags: u32,
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
    pub class_id: Option<u32>,
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
    pub perf: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub multipath: Option<Vec<MultipathRoute>>,
    // Missing support of RTA_NH_ID
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum AddressFamily {
    IPv4,
    IPv6,
    Other(u8),
    Unknown,
}

impl From<u8> for AddressFamily {
    fn from(d: u8) -> Self {
        match d {
            AF_INET => AddressFamily::IPv4,
            AF_INET6 => AddressFamily::IPv6,
            _ => AddressFamily::Other(d),
        }
    }
}

impl Default for AddressFamily {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RouteProtocol {
    UnSpec,
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

const RTPROT_UNSPEC: u8 = 0;
const RTPROT_REDIRECT: u8 = 1;
const RTPROT_KERNEL: u8 = 2;
const RTPROT_BOOT: u8 = 3;
const RTPROT_STATIC: u8 = 4;
const RTPROT_GATED: u8 = 8;
const RTPROT_RA: u8 = 9;
const RTPROT_MRT: u8 = 10;
const RTPROT_ZEBRA: u8 = 11;
const RTPROT_BIRD: u8 = 12;
const RTPROT_DNROUTED: u8 = 13;
const RTPROT_XORP: u8 = 14;
const RTPROT_NTK: u8 = 15;
const RTPROT_DHCP: u8 = 16;
const RTPROT_MROUTED: u8 = 17;
const RTPROT_KEEPALIVED: u8 = 18;
const RTPROT_BABEL: u8 = 42;
const RTPROT_BGP: u8 = 186;
const RTPROT_ISIS: u8 = 187;
const RTPROT_OSPF: u8 = 188;
const RTPROT_RIP: u8 = 189;
const RTPROT_EIGRP: u8 = 192;

impl From<u8> for RouteProtocol {
    fn from(d: u8) -> Self {
        match d {
            RTPROT_UNSPEC => RouteProtocol::UnSpec,
            RTPROT_REDIRECT => RouteProtocol::IcmpRedirect,
            RTPROT_KERNEL => RouteProtocol::Kernel,
            RTPROT_BOOT => RouteProtocol::Boot,
            RTPROT_STATIC => RouteProtocol::Static,
            RTPROT_GATED => RouteProtocol::Gated,
            RTPROT_RA => RouteProtocol::Ra,
            RTPROT_MRT => RouteProtocol::Mrt,
            RTPROT_ZEBRA => RouteProtocol::Zebra,
            RTPROT_BIRD => RouteProtocol::Bird,
            RTPROT_DNROUTED => RouteProtocol::DnRouted,
            RTPROT_XORP => RouteProtocol::Xorp,
            RTPROT_NTK => RouteProtocol::Ntk,
            RTPROT_DHCP => RouteProtocol::Dhcp,
            RTPROT_MROUTED => RouteProtocol::Mrouted,
            RTPROT_KEEPALIVED => RouteProtocol::KeepAlived,
            RTPROT_BABEL => RouteProtocol::Babel,
            RTPROT_BGP => RouteProtocol::Bgp,
            RTPROT_ISIS => RouteProtocol::Isis,
            RTPROT_OSPF => RouteProtocol::Ospf,
            RTPROT_RIP => RouteProtocol::Rip,
            RTPROT_EIGRP => RouteProtocol::Eigrp,
            _ => RouteProtocol::Other(d),
        }
    }
}

impl Default for RouteProtocol {
    fn default() -> Self {
        Self::Unknown
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
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RouteScope {
    Universe,
    Site,
    Link,
    Host,
    #[serde(rename = "no_where")]
    NoWhere,
    Unknown,
    Other(u8),
}

impl From<u8> for RouteScope {
    fn from(d: u8) -> Self {
        match d {
            RT_SCOPE_UNIVERSE => RouteScope::Universe,
            RT_SCOPE_SITE => RouteScope::Site,
            RT_SCOPE_LINK => RouteScope::Link,
            RT_SCOPE_HOST => RouteScope::Host,
            RT_SCOPE_NOWHERE => RouteScope::NoWhere,
            _ => RouteScope::Other(d),
        }
    }
}

impl Default for RouteScope {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RouteType {
    UnSpec,
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

impl From<u8> for RouteType {
    fn from(d: u8) -> Self {
        match d {
            RTN_UNSPEC => RouteType::UnSpec,
            RTN_UNICAST => RouteType::Unicast,
            RTN_LOCAL => RouteType::Local,
            RTN_BROADCAST => RouteType::Broadcast,
            RTN_ANYCAST => RouteType::Anycast,
            RTN_MULTICAST => RouteType::Multicast,
            RTN_BLACKHOLE => RouteType::BlackHole,
            RTN_UNREACHABLE => RouteType::Unreachable,
            RTN_PROHIBIT => RouteType::Prohibit,
            RTN_THROW => RouteType::Throw,
            RTN_NAT => RouteType::Nat,
            RTN_XRESOLVE => RouteType::ExternalResolve,
            _ => RouteType::Other(d),
        }
    }
}

impl Default for RouteType {
    fn default() -> Self {
        Self::Unknown
    }
}

const SIZE_OF_RTNEXTHOP: usize = 8;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub struct MultipathRoute {
    pub via: String,
    pub iface: String,
    pub weight: u16, // The kernel is u8, but ip route show it after + 1.
    pub flags: Vec<MultipathRouteFlags>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
    Other(u8),
}

const RTNH_F_DEAD: u8 = 1; /* Nexthop is dead (used by multipath) */
const RTNH_F_PERVASIVE: u8 = 2; /* Do recursive gateway lookup */
const RTNH_F_ONLINK: u8 = 4; /* Gateway is forced on link */
const RTNH_F_OFFLOAD: u8 = 8; /* offloaded route */
const RTNH_F_LINKDOWN: u8 = 16; /* carrier-down on nexthop */
const RTNH_F_UNRESOLVED: u8 = 32; /* The entry is unresolved (ipmr) */

pub(crate) async fn get_routes(
    ifaces: &HashMap<String, Iface>,
) -> Result<Vec<Route>, NisporError> {
    let mut routes = Vec::new();
    let mut ifindex_to_name = HashMap::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    for iface in ifaces.values() {
        ifindex_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }

    let mut links = handle.route().get(IpVersion::V6).execute();
    while let Some(rt_msg) = links.try_next().await? {
        routes.push(get_route(rt_msg, &ifindex_to_name)?);
    }
    let mut links = handle.route().get(IpVersion::V4).execute();
    while let Some(rt_msg) = links.try_next().await? {
        routes.push(get_route(rt_msg, &ifindex_to_name)?);
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
    rt.flags = header.flags.bits();
    rt.route_type = header.kind.into();
    let family = &rt.address_family;
    for nla in &route_msg.nlas {
        match nla {
            Nla::Destination(ref d) => {
                rt.dst = Some(format!(
                    "{}/{}",
                    _addr_to_string(d, family),
                    dst_prefix_len
                ));
            }
            Nla::Oif(ref d) => {
                rt.oif = if let Some(iface_name) =
                    ifindex_to_name.get(&format!("{}", d))
                {
                    Some(iface_name.clone())
                } else {
                    Some(format!("{}", d))
                }
            }
            Nla::PrefSource(ref d) => {
                rt.prefered_src = Some(_addr_to_string(d, family));
            }
            Nla::Table(d) => {
                rt.table = *d;
            }
            Nla::Flow(d) => {
                rt.class_id = Some(*d);
            }
            Nla::Source(ref d) => {
                rt.src = Some(format!(
                    "{}/{}",
                    _addr_to_string(d, family),
                    src_prefix_len
                ));
            }
            Nla::Gateway(ref d) => {
                rt.gateway = Some(_addr_to_string(d, family));
            }
            Nla::Via(ref d) => {
                rt.via = Some(_addr_to_string(d, family));
            }
            Nla::Metrics(ref d) => {
                let nlas = NlasIterator::new(d);
                for nla in nlas {
                    let metric = Metrics::parse(&nla?)?;
                    match metric {
                        Metrics::Lock(d) => {
                            rt.lock = Some(d);
                        }
                        Metrics::Mtu(d) => {
                            rt.mtu = Some(d);
                        }
                        Metrics::Window(d) => {
                            rt.window = Some(d);
                        }
                        Metrics::Rtt(d) => {
                            rt.rtt = Some(d);
                        }
                        Metrics::RttVar(d) => {
                            rt.rttvar = Some(d);
                        }
                        Metrics::SsThresh(d) => {
                            rt.ssthresh = Some(d);
                        }
                        Metrics::Cwnd(d) => {
                            rt.cwnd = Some(d);
                        }
                        Metrics::Advmss(d) => {
                            rt.advmss = Some(d);
                        }
                        Metrics::Reordering(d) => {
                            rt.reordering = Some(d);
                        }
                        Metrics::Hoplimit(d) => {
                            rt.hoplimit = Some(d);
                        }
                        Metrics::InitCwnd(d) => {
                            rt.initcwnd = Some(d);
                        }
                        Metrics::Features(d) => {
                            rt.features = Some(d);
                        }
                        Metrics::RtoMin(d) => {
                            rt.rto_min = Some(d);
                        }
                        Metrics::InitRwnd(d) => {
                            rt.initrwnd = Some(d);
                        }
                        Metrics::QuickAck(d) => {
                            rt.quickack = Some(d);
                        }
                        Metrics::CcAlgo(d) => {
                            rt.cc_algo = Some(d);
                        }
                        Metrics::FastopenNoCookie(d) => {
                            rt.fastopen_no_cookie = Some(d);
                        }
                        _ => {
                            eprintln!(
                                "Unknown RTA_METRICS message {:?}",
                                metric
                            );
                        }
                    }
                }
            }

            Nla::Mark(d) => {
                rt.mark = Some(*d);
            }
            Nla::Uid(d) => {
                rt.uid = Some(parse_as_u32(d)?);
            }
            Nla::Iif(d) => {
                rt.iif = if let Some(iface_name) =
                    ifindex_to_name.get(&format!("{}", d))
                {
                    Some(iface_name.clone())
                } else {
                    Some(format!("{}", d))
                }
            }
            Nla::CacheInfo(ref d) => {
                let cache_info = CacheInfo::parse(&CacheInfoBuffer::new(d))?;
                rt.cache_clntref = Some(cache_info.clntref);
                rt.cache_last_use = Some(cache_info.last_use);
                rt.cache_expires = Some(cache_info.expires / USER_HZ);
                rt.cache_error = Some(cache_info.error);
                rt.cache_used = Some(cache_info.used);
                rt.cache_id = Some(cache_info.id);
                rt.cache_ts = Some(cache_info.ts);
                rt.cache_ts_age = Some(cache_info.ts_age);
            }
            Nla::Priority(d) => {
                rt.metric = Some(*d);
            }
            Nla::MultiPath(d) => {
                let mut next_hops = Vec::new();
                let len = d.len();
                let mut i = 0usize;
                while (i < len) && (len - i > SIZE_OF_RTNEXTHOP) {
                    let nex_hop_len = parse_as_u16(&[
                        *d.get(i).ok_or(NisporError::bug(
                            "wrong index at multipath next_hop_len".into(),
                        ))?,
                        *d.get(i + 1).ok_or(NisporError::bug(
                            "wrong index at multipath next_hop_len".into(),
                        ))?,
                    ])?;
                    let nla = NlaBuffer::new(
                        &d[i + SIZE_OF_RTNEXTHOP..i + nex_hop_len as usize],
                    );
                    let via = match nla.kind() {
                        RTA_GATEWAY => _addr_to_string(nla.value(), family),
                        RTA_VIA => {
                            // Kernel will use RTA_VIA when gateway family does
                            // not match nexthop family
                            eprintln!(
                                "dual stack(RTA_VIA) multipath route next hop
                                 is not supported by nispor yet"
                            );
                            continue;
                        }
                        _ => {
                            eprintln!(
                                "Got unexpected RTA_MULTIPATH NLA {} {:?}",
                                nla.kind(),
                                nla.value()
                            );
                            continue;
                        }
                    };
                    let iface_index = parse_as_i32(
                        &d.get(i + 4..i + 8).ok_or(NisporError::bug(
                            "wrong index at multipath iface_index".into(),
                        ))?,
                    )?;
                    let iface = if let Some(iface_name) =
                        ifindex_to_name.get(&format!("{}", iface_index))
                    {
                        iface_name.clone()
                    } else {
                        format!("{}", iface_index)
                    };
                    let mut flags = Vec::new();
                    let flags_raw = d.get(i + 2).ok_or(NisporError::bug(
                        "wrong index at flags raw".into(),
                    ))?;
                    //TODO: Need better way to handle the bitmap.
                    if (flags_raw & RTNH_F_DEAD) > 0 {
                        flags.push(MultipathRouteFlags::Dead);
                    } else if (flags_raw & RTNH_F_PERVASIVE) > 0 {
                        flags.push(MultipathRouteFlags::Pervasive);
                    } else if (flags_raw & RTNH_F_ONLINK) > 0 {
                        flags.push(MultipathRouteFlags::OnLink);
                    } else if (flags_raw & RTNH_F_OFFLOAD) > 0 {
                        flags.push(MultipathRouteFlags::Offload);
                    } else if (flags_raw & RTNH_F_LINKDOWN) > 0 {
                        flags.push(MultipathRouteFlags::LinkDown);
                    } else if (flags_raw & RTNH_F_UNRESOLVED) > 0 {
                        flags.push(MultipathRouteFlags::Unresolved);
                    }

                    let next_hop = MultipathRoute {
                        flags: flags,
                        weight: *d.get(i + 3).ok_or(NisporError::bug(
                            "wrong index at weight".into(),
                        ))? as u16
                            + 1,
                        iface: iface,
                        via: via,
                    };
                    next_hops.push(next_hop);
                    i += nex_hop_len as usize;
                }
                rt.multipath = Some(next_hops);
            }
            Nla::Pref(d) => {
                rt.perf = Some(d[0]);
            }
            _ => eprintln!("Unknown NLA message for route {:?}", nla),
        }
    }

    Ok(rt)
}

fn _addr_to_string(data: &[u8], family: &AddressFamily) -> String {
    match family {
        AddressFamily::IPv4 => parse_as_ipv4(data).to_string(),
        AddressFamily::IPv6 => parse_as_ipv6(data).to_string(),
        _ => format!("{:?}", data),
    }
}
