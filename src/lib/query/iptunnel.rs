// SPDX-License-Identifier: Apache-2.0
use std::{collections::HashMap, net::IpAddr};

use rtnetlink::packet_route::{
    IpProtocol,
    link::{self, InfoData, InfoIpTunnel},
};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TunnelEncapFlag {
    CSum,
    CSum6,
    RemCSum,
    Other(u16),
}

impl TunnelEncapFlag {
    pub(crate) fn from_netlink(d: link::TunnelEncapFlags) -> Vec<Self> {
        d.iter()
            .map(|bit| match bit {
                link::TunnelEncapFlags::CSum => Self::CSum,
                link::TunnelEncapFlags::CSum6 => Self::CSum6,
                link::TunnelEncapFlags::RemCSum => Self::RemCSum,
                _ => Self::Other(bit.bits()),
            })
            .collect()
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum TunnelEncapType {
    None,
    Fou,
    Gue,
    Mpls,
    Other(u16),
}

impl From<TunnelEncapType> for rtnetlink::packet_route::link::TunnelEncapType {
    fn from(d: TunnelEncapType) -> Self {
        match d {
            TunnelEncapType::None => Self::None,
            TunnelEncapType::Fou => Self::Fou,
            TunnelEncapType::Gue => Self::Gue,
            TunnelEncapType::Mpls => Self::Mpls,
            TunnelEncapType::Other(d) => Self::Other(d),
        }
    }
}

impl From<link::TunnelEncapType> for TunnelEncapType {
    fn from(d: link::TunnelEncapType) -> Self {
        match d {
            link::TunnelEncapType::None => Self::None,
            link::TunnelEncapType::Fou => Self::Fou,
            link::TunnelEncapType::Gue => Self::Gue,
            link::TunnelEncapType::Mpls => Self::Mpls,
            link::TunnelEncapType::Other(d) => Self::Other(d),
            _ => Self::Other(d.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Ip6TunnelFlag {
    IgnEncapLimit,
    UseOrigTclass,
    UseOrigFlowlabel,
    Mip6Dev,
    RcvDscpCopy,
    UseOrigFwMark,
    AllowLocalRemote,
    CapXmit,
    CapRcv,
    CapPerPacket,
    Other(u32),
}

impl Ip6TunnelFlag {
    pub(crate) fn from_netlink(d: link::Ip6TunnelFlags) -> Vec<Self> {
        d.iter()
            .map(|bit| match bit {
                link::Ip6TunnelFlags::IgnEncapLimit => Self::IgnEncapLimit,
                link::Ip6TunnelFlags::UseOrigTclass => Self::UseOrigTclass,
                link::Ip6TunnelFlags::UseOrigFlowlabel => {
                    Self::UseOrigFlowlabel
                }
                link::Ip6TunnelFlags::Mip6Dev => Self::Mip6Dev,
                link::Ip6TunnelFlags::RcvDscpCopy => Self::RcvDscpCopy,
                link::Ip6TunnelFlags::UseOrigFwMark => Self::UseOrigFwMark,
                link::Ip6TunnelFlags::AllowLocalRemote => {
                    Self::AllowLocalRemote
                }
                link::Ip6TunnelFlags::CapXmit => Self::CapXmit,
                link::Ip6TunnelFlags::CapRcv => Self::CapRcv,
                link::Ip6TunnelFlags::CapPerPacket => Self::CapPerPacket,
                _ => Self::Other(bit.bits()),
            })
            .collect()
    }
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum IpTunnelMode {
    #[default]
    Unknown,
    Ipip,
    Sit,
    Ip6ip6,
    Ipip6,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct IpTunnelInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<IpAddr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,

    pub mode: IpTunnelMode,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub tos: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ip6tun_flags: Option<Vec<Ip6TunnelFlag>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pmtu_disc: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encap_limit: Option<u8>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow_info: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encap_type: Option<TunnelEncapType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encap_flags: Option<Vec<TunnelEncapFlag>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encap_source_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub encap_destination_port: Option<u16>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub collect_metadata: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fwmark: Option<u32>,

    #[serde(skip_serializing)]
    _parent_ifindex: Option<u32>,
}

pub(crate) fn get_ip_tunnel_info(
    data: &InfoData,
    if_type: IfaceType,
) -> Option<IpTunnelInfo> {
    let mut ip_tunnel_info = IpTunnelInfo::default();
    let mut protocol: Option<IpProtocol> = None;

    if let InfoData::IpTunnel(infos) = data {
        for info in infos {
            match *info {
                InfoIpTunnel::Local(d) => {
                    ip_tunnel_info.local = Some(d);
                }
                InfoIpTunnel::Remote(d) => {
                    ip_tunnel_info.remote = Some(d);
                }
                InfoIpTunnel::Link(d) => {
                    ip_tunnel_info._parent_ifindex = Some(d);
                }
                InfoIpTunnel::Ttl(d) => {
                    ip_tunnel_info.ttl = Some(d);
                }
                InfoIpTunnel::Tos(d) => {
                    ip_tunnel_info.tos = Some(d);
                }
                InfoIpTunnel::FlowInfo(d) => {
                    ip_tunnel_info.flow_info = Some(d);
                }
                InfoIpTunnel::Ipv6Flags(d) => {
                    ip_tunnel_info.ip6tun_flags =
                        Some(Ip6TunnelFlag::from_netlink(d));
                }
                InfoIpTunnel::Protocol(d) => {
                    protocol = Some(d);
                }
                InfoIpTunnel::PMtuDisc(d) => {
                    ip_tunnel_info.pmtu_disc = Some(d);
                }
                InfoIpTunnel::EncapLimit(d) => {
                    ip_tunnel_info.encap_limit = Some(d);
                }
                InfoIpTunnel::EncapType(d) => {
                    ip_tunnel_info.encap_type = Some(d.into());
                }
                InfoIpTunnel::EncapFlags(d) => {
                    ip_tunnel_info.encap_flags =
                        Some(TunnelEncapFlag::from_netlink(d));
                }
                InfoIpTunnel::EncapSPort(d) => {
                    ip_tunnel_info.encap_source_port = Some(d);
                }
                InfoIpTunnel::EncapDPort(d) => {
                    ip_tunnel_info.encap_destination_port = Some(d);
                }
                InfoIpTunnel::CollectMetadata(d) => {
                    ip_tunnel_info.collect_metadata = Some(d);
                }
                InfoIpTunnel::FwMark(d) => {
                    ip_tunnel_info.fwmark = Some(d);
                }
                _ => {}
            }
        }

        ip_tunnel_info.mode = match (protocol, if_type) {
            (_, IfaceType::SitTun) => IpTunnelMode::Sit,
            (_, IfaceType::Ipip) => IpTunnelMode::Ipip,
            (Some(IpProtocol::Ipip), IfaceType::Ip6Tnl) => IpTunnelMode::Ipip6,
            (Some(IpProtocol::Ipv6), IfaceType::Ip6Tnl) => IpTunnelMode::Ip6ip6,
            _ => IpTunnelMode::Unknown,
        };

        Some(ip_tunnel_info)
    } else {
        None
    }
}

pub(crate) fn ip_tunnel_iface_tidy_up(
    iface_states: &mut HashMap<String, Iface>,
) {
    fill_port_iface_names(iface_states);
}

fn fill_port_iface_names(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(iface.index, iface.name.clone());
    }

    for iface in iface_states.values_mut() {
        if let Some(tun) = iface.ip_tunnel.as_mut()
            && let Some(parent_iface_name) =
                index_to_name.get(&tun._parent_ifindex.unwrap_or_default())
        {
            tun.parent = Some(parent_iface_name.to_string());
        }
    }
}
