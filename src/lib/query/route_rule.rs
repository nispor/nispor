// SPDX-License-Identifier: Apache-2.0

use futures::stream::TryStreamExt;
use netlink_packet_route::route::RouteHeader;
use netlink_packet_route::rule::{self, RuleAttribute, RuleMessage};

use rtnetlink::new_connection;
use rtnetlink::IpVersion;
use serde::{Deserialize, Serialize};

use crate::{AddressFamily, NisporError, RouteProtocol};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
pub enum RuleAction {
    #[default]
    Unspec,
    /* Pass to fixed table or l3mdev */
    Table,
    /* Jump to another rule */
    Goto,
    /* No operation */
    Nop,
    /* Drop without notification */
    Blackhole,
    /* Drop with ENETUNREACH */
    Unreachable,
    /* Drop with EACCES */
    Prohibit,
    Other(u8),
}

impl From<rule::RuleAction> for RuleAction {
    fn from(d: rule::RuleAction) -> Self {
        match d {
            rule::RuleAction::Unspec => Self::Unspec,
            rule::RuleAction::ToTable => Self::Table,
            rule::RuleAction::Goto => Self::Goto,
            rule::RuleAction::Nop => Self::Nop,
            rule::RuleAction::Blackhole => Self::Blackhole,
            rule::RuleAction::Unreachable => Self::Unreachable,
            rule::RuleAction::Prohibit => Self::Prohibit,
            _ => Self::Other(d.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct RouteRule {
    pub action: RuleAction,
    pub address_family: AddressFamily,
    pub flags: u32,
    pub tos: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub table: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iif: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oif: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub goto: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fw_mark: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fw_mask: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm: Option<RouteRealm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tun_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_ifgroup: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_prefix_len: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<RouteProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip_proto: Option<IpProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_port_range: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dst_port_range: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub l3mdev: Option<bool>,
}

pub(crate) async fn get_route_rules() -> Result<Vec<RouteRule>, NisporError> {
    let mut rules = Vec::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle.rule().get(IpVersion::V6).execute();
    while let Some(rt_msg) = links.try_next().await? {
        rules.push(get_rule(rt_msg)?);
    }
    let mut links = handle.rule().get(IpVersion::V4).execute();
    while let Some(rt_msg) = links.try_next().await? {
        rules.push(get_rule(rt_msg)?);
    }
    Ok(rules)
}

fn get_rule(rule_msg: RuleMessage) -> Result<RouteRule, NisporError> {
    let mut rl = RouteRule::default();
    let header = &rule_msg.header;
    rl.address_family = header.family.into();
    let src_prefix_len = header.src_len;
    let dst_prefix_len = header.dst_len;
    rl.tos = header.tos;
    rl.action = header.action.into();
    if header.table > RouteHeader::RT_TABLE_UNSPEC {
        rl.table = Some(header.table.into());
    }
    let _family = &rl.address_family;
    for nla in &rule_msg.attributes {
        match nla {
            RuleAttribute::Destination(d) => {
                rl.dst = Some(format!("{}/{}", d, dst_prefix_len,));
            }
            RuleAttribute::Source(d) => {
                rl.src = Some(format!("{}/{}", d, src_prefix_len,));
            }
            RuleAttribute::Iifname(d) => {
                rl.iif = Some(d.clone().to_string());
            }
            RuleAttribute::Oifname(d) => {
                rl.oif = Some(d.clone().to_string());
            }
            RuleAttribute::Goto(d) => {
                rl.goto = Some(*d);
            }
            RuleAttribute::Priority(d) => {
                rl.priority = Some(*d);
            }
            RuleAttribute::FwMark(d) => {
                rl.fw_mark = Some(*d);
            }
            RuleAttribute::FwMask(d) => {
                rl.fw_mask = Some(*d);
            }
            RuleAttribute::Realm(d) => {
                rl.realm = Some((*d).into());
            }
            RuleAttribute::TunId(d) => {
                rl.tun_id = Some(*d);
            }
            RuleAttribute::SuppressIfGroup(d) => {
                if *d != std::u32::MAX {
                    rl.suppress_ifgroup = Some(*d);
                }
            }
            RuleAttribute::SuppressPrefixLen(d) => {
                if *d != std::u32::MAX {
                    rl.suppress_prefix_len = Some(*d);
                }
            }
            RuleAttribute::Table(d) => {
                if *d > RouteHeader::RT_TABLE_UNSPEC.into() {
                    rl.table = Some(*d);
                }
            }
            RuleAttribute::Protocol(d) => {
                rl.protocol = Some((*d).into());
            }
            RuleAttribute::IpProtocol(d) => {
                rl.ip_proto = Some((*d).into());
            }
            RuleAttribute::L3MDev(d) => {
                rl.l3mdev = Some(*d);
            }
            _ => log::debug!("Unknown NLA message for route rule {:?}", nla),
        }
    }

    Ok(rl)
}

#[derive(
    Debug, PartialEq, Eq, Clone, Copy, Default, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IpProtocol {
    Hopopts,
    Icmp,
    Igmp,
    Ipip,
    Tcp,
    Egp,
    Pup,
    Udp,
    Idp,
    Tp,
    Dccp,
    Ipv6,
    Rsvp,
    Gre,
    Esp,
    Ah,
    Mtp,
    Beetph,
    Encap,
    Pim,
    Comp,
    L2tp,
    Sctp,
    Udplite,
    Mpls,
    Ethernet,
    #[default]
    Raw,
    Mptcp,
    Other(i32),
}

impl From<netlink_packet_route::IpProtocol> for IpProtocol {
    fn from(d: netlink_packet_route::IpProtocol) -> Self {
        match d {
            netlink_packet_route::IpProtocol::Hopopts => Self::Hopopts,
            netlink_packet_route::IpProtocol::Icmp => Self::Icmp,
            netlink_packet_route::IpProtocol::Igmp => Self::Igmp,
            netlink_packet_route::IpProtocol::Ipip => Self::Ipip,
            netlink_packet_route::IpProtocol::Tcp => Self::Tcp,
            netlink_packet_route::IpProtocol::Egp => Self::Egp,
            netlink_packet_route::IpProtocol::Pup => Self::Pup,
            netlink_packet_route::IpProtocol::Udp => Self::Udp,
            netlink_packet_route::IpProtocol::Idp => Self::Idp,
            netlink_packet_route::IpProtocol::Tp => Self::Tp,
            netlink_packet_route::IpProtocol::Dccp => Self::Dccp,
            netlink_packet_route::IpProtocol::Ipv6 => Self::Ipv6,
            netlink_packet_route::IpProtocol::Rsvp => Self::Rsvp,
            netlink_packet_route::IpProtocol::Gre => Self::Gre,
            netlink_packet_route::IpProtocol::Esp => Self::Esp,
            netlink_packet_route::IpProtocol::Ah => Self::Ah,
            netlink_packet_route::IpProtocol::Mtp => Self::Mtp,
            netlink_packet_route::IpProtocol::Beetph => Self::Beetph,
            netlink_packet_route::IpProtocol::Encap => Self::Encap,
            netlink_packet_route::IpProtocol::Pim => Self::Pim,
            netlink_packet_route::IpProtocol::Comp => Self::Comp,
            netlink_packet_route::IpProtocol::L2tp => Self::L2tp,
            netlink_packet_route::IpProtocol::Sctp => Self::Sctp,
            netlink_packet_route::IpProtocol::Udplite => Self::Udplite,
            netlink_packet_route::IpProtocol::Mpls => Self::Mpls,
            netlink_packet_route::IpProtocol::Ethernet => Self::Ethernet,
            netlink_packet_route::IpProtocol::Raw => Self::Raw,
            netlink_packet_route::IpProtocol::Mptcp => Self::Mptcp,
            _ => Self::Other(d.into()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Copy, Serialize, Deserialize)]
pub struct RouteRealm {
    pub source: u16,
    pub destination: u16,
}

impl From<netlink_packet_route::route::RouteRealm> for RouteRealm {
    fn from(d: netlink_packet_route::route::RouteRealm) -> Self {
        Self {
            source: d.source,
            destination: d.destination,
        }
    }
}
