// SPDX-License-Identifier: Apache-2.0

use std::net::{IpAddr, Ipv6Addr};
use std::str::FromStr;

use netlink_packet_route::rtnl::link::nlas::{AfSpecInet, Inet6};

use serde::{Deserialize, Serialize};

use crate::{Iface, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Ipv4Info {
    pub addresses: Vec<Ipv4AddrInfo>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Ipv4AddrInfo {
    pub address: String,
    pub prefix_len: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    // The renaming seonds for this address be valid
    pub valid_lft: String,
    // The renaming seonds for this address be preferred
    pub preferred_lft: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Ipv6Info {
    pub addresses: Vec<Ipv6AddrInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Ipv6AddrInfo {
    pub address: String,
    pub prefix_len: u8,
    // The renaming seonds for this address be valid
    pub valid_lft: String,
    // The renaming seonds for this address be preferred
    pub preferred_lft: String,
    /// IPv6 Address Flags
    pub flags: Vec<Ipv6AddrFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<Ipv6Addr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_prefix_len: Option<u8>,
}

pub(crate) fn parse_ip_addr_str(
    ip_addr_str: &str,
) -> Result<IpAddr, NisporError> {
    IpAddr::from_str(ip_addr_str).map_err(|e| {
        let e = NisporError::invalid_argument(format!(
            "Invalid IP address {ip_addr_str}: {e}"
        ));
        log::error!("{}", e);
        e
    })
}

pub(crate) fn parse_ip_net_addr_str(
    ip_net_str: &str,
) -> Result<(IpAddr, u8), NisporError> {
    let splits: Vec<&str> = ip_net_str.split('/').collect();
    if splits.len() > 2 || splits.is_empty() {
        let e = NisporError::invalid_argument(format!(
            "Invalid IP network address {ip_net_str}",
        ));
        log::error!("{}", e);
        return Err(e);
    }
    let addr_str = splits[0];
    let prefix_len = if let Some(prefix_len_str) = splits.get(1) {
        prefix_len_str.parse::<u8>().map_err(|e| {
            let e = NisporError::invalid_argument(format!(
                "Invalid IP network prefix {ip_net_str}: {e}"
            ));
            log::error!("{}", e);
            e
        })?
    } else if is_ipv6_addr(addr_str) {
        128
    } else {
        32
    };
    Ok((parse_ip_addr_str(addr_str)?, prefix_len))
}

pub(crate) fn fill_af_spec_inet_info(iface: &mut Iface, nlas: &[AfSpecInet]) {
    for nla in nlas {
        if let AfSpecInet::Inet6(nlas) = nla {
            for nla in nlas {
                if let Inet6::Token(raw) = nla {
                    // Kernel set all zero as default value
                    if raw != &[0; 16] {
                        if iface.ipv6.is_none() {
                            iface.ipv6 = Some(Ipv6Info::default());
                        }
                        if let Some(ipv6_info) = iface.ipv6.as_mut() {
                            ipv6_info.token = Some(ipv6_token_to_string(*raw));
                        }
                    }
                }
            }
        }
    }
}

// The Ipv6Addr::to_string() will convert
//  ::fac1 to ::0.0.250.193
// Which is no ideal in this case
// To workaround that, we set leading 64 bites to '2001:db8::', and
// then trip it out from string.
fn ipv6_token_to_string(raw: [u8; 16]) -> String {
    let token = Ipv6Addr::from(raw);
    let mut segments = token.segments();
    segments[0] = 0x2001;
    segments[1] = 0xdb8;
    Ipv6Addr::from(segments).to_string()["2001:db8".len()..].to_string()
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum IpFamily {
    Ipv4,
    Ipv6,
}

pub(crate) fn is_ipv6_addr(addr: &str) -> bool {
    addr.contains(':')
}

const IFA_F_SECONDARY: u32 = 0x01;
const IFA_F_NODAD: u32 = 0x02;
const IFA_F_OPTIMISTIC: u32 = 0x04;
const IFA_F_DADFAILED: u32 = 0x08;
const IFA_F_HOMEADDRESS: u32 = 0x10;
const IFA_F_DEPRECATED: u32 = 0x20;
const IFA_F_TENTATIVE: u32 = 0x40;
const IFA_F_PERMANENT: u32 = 0x80;
const IFA_F_MANAGETEMPADDR: u32 = 0x100;
const IFA_F_NOPREFIXROUTE: u32 = 0x200;
const IFA_F_MCAUTOJOIN: u32 = 0x400;
const IFA_F_STABLE_PRIVACY: u32 = 0x800;

#[derive(Clone, Eq, PartialEq, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
#[repr(u32)]
pub enum Ipv6AddrFlag {
    Secondary = IFA_F_SECONDARY,
    Nodad = IFA_F_NODAD,
    Optimistic = IFA_F_OPTIMISTIC,
    Dadfailed = IFA_F_DADFAILED,
    Homeaddress = IFA_F_HOMEADDRESS,
    Deprecated = IFA_F_DEPRECATED,
    Tentative = IFA_F_TENTATIVE,
    Permanent = IFA_F_PERMANENT,
    Managetempaddr = IFA_F_MANAGETEMPADDR,
    Noprefixroute = IFA_F_NOPREFIXROUTE,
    Mcautojoin = IFA_F_MCAUTOJOIN,
    StablePrivacy = IFA_F_STABLE_PRIVACY,
}

impl Ipv6AddrFlag {
    pub(crate) fn all() -> [Ipv6AddrFlag; 12] {
        [
            Ipv6AddrFlag::Secondary,
            Ipv6AddrFlag::Nodad,
            Ipv6AddrFlag::Optimistic,
            Ipv6AddrFlag::Dadfailed,
            Ipv6AddrFlag::Homeaddress,
            Ipv6AddrFlag::Deprecated,
            Ipv6AddrFlag::Tentative,
            Ipv6AddrFlag::Permanent,
            Ipv6AddrFlag::Managetempaddr,
            Ipv6AddrFlag::Noprefixroute,
            Ipv6AddrFlag::Mcautojoin,
            Ipv6AddrFlag::StablePrivacy,
        ]
    }
}
