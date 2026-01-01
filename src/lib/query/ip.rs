// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
    net::{IpAddr, Ipv6Addr},
    str::FromStr,
};

use rtnetlink::packet_route::{
    address,
    address::{AddressAttribute, AddressMessage},
    link::{AfSpecInet6, AfSpecUnspec},
};
use serde::{Deserialize, Serialize};

use crate::{Iface, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Ipv4Info {
    pub addresses: Vec<Ipv4AddrInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forwarding: Option<bool>,
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
        log::error!("{e}");
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
        log::error!("{e}");
        return Err(e);
    }
    let addr_str = splits[0];
    let prefix_len = if let Some(prefix_len_str) = splits.get(1) {
        prefix_len_str.parse::<u8>().map_err(|e| {
            let e = NisporError::invalid_argument(format!(
                "Invalid IP network prefix {ip_net_str}: {e}"
            ));
            log::error!("{e}");
            e
        })?
    } else if is_ipv6_addr(addr_str) {
        128
    } else {
        32
    };
    Ok((parse_ip_addr_str(addr_str)?, prefix_len))
}

pub(crate) fn fill_af_spec_inet_info(iface: &mut Iface, nlas: &[AfSpecUnspec]) {
    for nla in nlas {
        if let AfSpecUnspec::Inet6(nlas) = nla {
            for nla in nlas {
                if let AfSpecInet6::Token(addr) = nla {
                    // Kernel set all zero as default value
                    if *addr != Ipv6Addr::UNSPECIFIED {
                        if iface.ipv6.is_none() {
                            iface.ipv6 = Some(Ipv6Info::default());
                        }
                        if let Some(ipv6_info) = iface.ipv6.as_mut() {
                            ipv6_info.token = Some(ipv6_token_to_string(*addr));
                        }
                    }
                }
            }
        }
    }
}

fn read_ipv4_forwarding(iface_name: &str) -> Option<bool> {
    let path = format!("/proc/sys/net/ipv4/conf/{iface_name}/forwarding");

    let file = match File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            log::warn!(
                "Failed to read IPv4 forwarding value for interface \
                 '{iface_name}': could not open '{path}': {e}"
            );
            return None;
        }
    };

    let mut reader = BufReader::new(file);
    let mut line = String::new();

    if let Err(e) = reader.read_line(&mut line) {
        log::warn!(
            "Failed to read IPv4 forwarding value from '{path}', for \
             interface '{iface_name}': {e}"
        );
        return None;
    }

    match line.trim() {
        "1" => Some(true),
        "0" => Some(false),
        other => {
            log::warn!(
                "Unexpected IPv4 forwarding value '{other}' in '{path}', for \
                 interface '{iface_name}'"
            );
            None
        }
    }
}

// The Ipv6Addr::to_string() will convert
//  ::fac1 to ::0.0.250.193
// Which is no ideal in this case
// To workaround that, we set leading 64 bites to '2001:db8::', and
// then trip it out from string.
fn ipv6_token_to_string(addr: Ipv6Addr) -> String {
    let mut segments = addr.segments();
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

#[derive(Clone, Eq, PartialEq, Debug, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum Ipv6AddrFlag {
    Secondary,
    Nodad,
    Optimistic,
    Dadfailed,
    Homeaddress,
    Deprecated,
    Tentative,
    Permanent,
    Managetempaddr,
    Noprefixroute,
    Mcautojoin,
    StablePrivacy,
    Other(u32),
}

impl From<address::AddressFlags> for Ipv6AddrFlag {
    fn from(d: address::AddressFlags) -> Self {
        match d {
            address::AddressFlags::Secondary => Self::Secondary,
            address::AddressFlags::Nodad => Self::Nodad,
            address::AddressFlags::Optimistic => Self::Optimistic,
            address::AddressFlags::Dadfailed => Self::Dadfailed,
            address::AddressFlags::Homeaddress => Self::Homeaddress,
            address::AddressFlags::Deprecated => Self::Deprecated,
            address::AddressFlags::Tentative => Self::Tentative,
            address::AddressFlags::Permanent => Self::Permanent,
            address::AddressFlags::Managetempaddr => Self::Managetempaddr,
            address::AddressFlags::Noprefixroute => Self::Noprefixroute,
            address::AddressFlags::Mcautojoin => Self::Mcautojoin,
            address::AddressFlags::StablePrivacy => Self::StablePrivacy,
            _ => Self::Other(d.bits()),
        }
    }
}

pub(crate) fn fill_ip_forwarding(iface_states: &mut HashMap<String, Iface>) {
    for (iface_name, iface_state) in iface_states.iter_mut() {
        iface_state
            .ipv4
            .get_or_insert(Default::default())
            .forwarding = read_ipv4_forwarding(iface_name);
    }
}

pub(crate) fn fill_ip_addr(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &AddressMessage,
) -> Result<(), NisporError> {
    match nl_msg.header.family {
        rtnetlink::packet_route::AddressFamily::Inet => {
            let (iface_index, addr) = parse_ipv4_nlas(nl_msg)?;
            if let Some(i) = get_iface_name_by_index(iface_states, iface_index)
            {
                let iface_name = i.to_string();
                if let Some(iface) = iface_states.get_mut(iface_name.as_str()) {
                    if iface.ipv4.is_none() {
                        iface.ipv4 = Some(Ipv4Info::default());
                    }
                    if let Some(ipv4_info) = iface.ipv4.as_mut() {
                        ipv4_info.addresses.push(addr);
                    }
                }
            }
        }
        rtnetlink::packet_route::AddressFamily::Inet6 => {
            let (iface_index, addr) = parse_ipv6_nlas(nl_msg)?;
            if let Some(i) = get_iface_name_by_index(iface_states, iface_index)
            {
                let iface_name = i.to_string();
                if let Some(iface) = iface_states.get_mut(iface_name.as_str()) {
                    if iface.ipv6.is_none() {
                        iface.ipv6 = Some(Ipv6Info::default());
                    }
                    if let Some(ipv6_info) = iface.ipv6.as_mut() {
                        ipv6_info.addresses.push(addr);
                    }
                }
            }
        }
        _ => {
            log::warn!(
                "unknown address family {} {:?}",
                u8::from(nl_msg.header.family),
                nl_msg
            );
        }
    };
    Ok(())
}

// TODO: remove the dupcode between parse_ipv4_nlas() and parse_ipv6_nlas()
fn parse_ipv4_nlas(
    nl_msg: &AddressMessage,
) -> Result<(u32, Ipv4AddrInfo), NisporError> {
    let iface_index = nl_msg.header.index;
    let mut addr = Ipv4AddrInfo {
        prefix_len: nl_msg.header.prefix_len,
        ..Default::default()
    };
    let mut peer = String::new();
    for nla in &nl_msg.attributes {
        if let AddressAttribute::Local(v) = nla {
            addr.address = v.to_string();
        } else if let AddressAttribute::Address(v) = nla {
            peer = v.to_string();
        } else if let AddressAttribute::CacheInfo(v) = nla {
            addr.preferred_lft = left_time_to_string(v.ifa_preferred);
            addr.valid_lft = left_time_to_string(v.ifa_valid);
        }
    }

    if peer != addr.address {
        addr.peer = Some(peer)
    }

    Ok((iface_index, addr))
}

fn parse_ipv6_nlas(
    nl_msg: &AddressMessage,
) -> Result<(u32, Ipv6AddrInfo), NisporError> {
    let iface_index = nl_msg.header.index;
    let mut addr = Ipv6AddrInfo {
        prefix_len: nl_msg.header.prefix_len,
        ..Default::default()
    };

    for nla in &nl_msg.attributes {
        if let AddressAttribute::Local(v) = nla {
            addr.address = v.to_string();
            addr.peer_prefix_len = Some(addr.prefix_len);
            addr.prefix_len = 128;
        }
    }

    for nla in &nl_msg.attributes {
        if let AddressAttribute::Address(IpAddr::V6(v)) = nla {
            if addr.peer_prefix_len.is_some() {
                addr.peer = Some(*v);
            } else {
                addr.address = v.to_string();
            }
        } else if let AddressAttribute::CacheInfo(v) = nla {
            addr.preferred_lft = left_time_to_string(v.ifa_preferred);
            addr.valid_lft = left_time_to_string(v.ifa_valid);
        } else if let AddressAttribute::Flags(flags) = nla {
            addr.flags = flags.iter().map(Ipv6AddrFlag::from).collect();
        }
    }

    Ok((iface_index, addr))
}

fn left_time_to_string(left_time: u32) -> String {
    if left_time == u32::MAX {
        "forever".into()
    } else {
        format!("{left_time}sec")
    }
}

fn get_iface_name_by_index(
    iface_states: &HashMap<String, Iface>,
    iface_index: u32,
) -> Option<&str> {
    for (iface_name, iface) in iface_states.iter() {
        if iface.index == iface_index {
            return Some(iface_name.as_str());
        }
    }
    None
}
