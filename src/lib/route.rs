use crate::ifaces::Iface;
use crate::netlink::parse_as_ipv4;
use crate::netlink::parse_as_ipv6;
use crate::netlink::parse_as_u32;
use crate::netlink::AF_INET;
use crate::netlink::AF_INET6;
use crate::NisporError;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::nlas::route::CacheInfo;
use netlink_packet_route::rtnl::nlas::route::CacheInfoBuffer;
use netlink_packet_route::rtnl::nlas::route::Metrics;
use netlink_packet_route::rtnl::nlas::route::Nla;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use netlink_packet_route::rtnl::{
    RTN_ANYCAST, RTN_BLACKHOLE, RTN_BROADCAST, RTN_LOCAL, RTN_MULTICAST,
    RTN_NAT, RTN_PROHIBIT, RTN_THROW, RTN_UNICAST, RTN_UNREACHABLE, RTN_UNSPEC,
    RTN_XRESOLVE, RTPROT_BOOT, RTPROT_KERNEL, RTPROT_REDIRECT, RTPROT_STATIC,
    RTPROT_UNSPEC, RT_SCOPE_HOST, RT_SCOPE_LINK, RT_SCOPE_NOWHERE,
    RT_SCOPE_SITE, RT_SCOPE_UNIVERSE,
};
use netlink_packet_route::RouteMessage;
use netlink_packet_utils::traits::Parseable;
use rtnetlink::new_connection;
use rtnetlink::IpVersion;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::runtime::Runtime;

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
    // Missing support of RTA_NH_ID
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
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
pub enum RouteProtocol {
    UnSpec,
    IcmpRedirect,
    Kernel,
    Boot,
    Static,
    Unknown,
    Other(u8),
}

impl From<u8> for RouteProtocol {
    fn from(d: u8) -> Self {
        match d {
            RTPROT_UNSPEC => RouteProtocol::UnSpec,
            RTPROT_REDIRECT => RouteProtocol::IcmpRedirect,
            RTPROT_KERNEL => RouteProtocol::Kernel,
            RTPROT_BOOT => RouteProtocol::Boot,
            RTPROT_STATIC => RouteProtocol::Static,
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
pub enum RouteScope {
    Universe,
    Site,
    Link,
    Host,
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
    #[serde(rename = "NAT")]
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

pub(crate) fn get_routes(
    ifaces: &HashMap<String, Iface>,
) -> Result<Vec<Route>, NisporError> {
    let mut ifindex_to_name = HashMap::new();
    for iface in ifaces.values() {
        ifindex_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    Ok(Runtime::new()?.block_on(_get_routes(&ifindex_to_name))?)
}

async fn _get_routes(
    ifindex_to_name: &HashMap<String, String>,
) -> Result<Vec<Route>, NisporError> {
    let mut routes = Vec::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle.route().get(IpVersion::V6).execute();
    while let Some(rt_msg) = links.try_next().await? {
        routes.push(get_route(rt_msg, ifindex_to_name)?);
    }
    let mut links = handle.route().get(IpVersion::V4).execute();
    while let Some(rt_msg) = links.try_next().await? {
        routes.push(get_route(rt_msg, ifindex_to_name)?);
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
                rt.uid = Some(parse_as_u32(d));
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
            Nla::MultiPath(_) => {
                eprintln!(
                    "netlink-rust does not support parsing
                    multipath routes yet"
                );
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
        AddressFamily::IPv4 => parse_as_ipv4(data),
        AddressFamily::IPv6 => parse_as_ipv6(data),
        _ => format!("{:?}", data),
    }
}
