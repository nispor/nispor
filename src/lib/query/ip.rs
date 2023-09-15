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
