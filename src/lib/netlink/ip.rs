// SPDX-License-Identifier: Apache-2.0

use crate::{
    netlink::nla::{parse_as_ipv4, parse_as_ipv6},
    Iface, Ipv4AddrInfo, Ipv4Info, Ipv6AddrFlag, Ipv6AddrInfo, Ipv6Info,
    NisporError,
};
use netlink_packet_route::address::{CacheInfo, CacheInfoBuffer};
use netlink_packet_route::rtnl::address::nlas::Nla;
use netlink_packet_route::rtnl::AddressMessage;
use netlink_packet_utils::Parseable;
use std::collections::HashMap;

pub(crate) const AF_INET: u8 = 2;
pub(crate) const AF_INET6: u8 = 10;

pub(crate) fn fill_ip_addr(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &AddressMessage,
) -> Result<(), NisporError> {
    match nl_msg.header.family {
        AF_INET => {
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
        AF_INET6 => {
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
                nl_msg.header.family,
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
    for nla in &nl_msg.nlas {
        if let Nla::Local(addr_vec) = nla {
            addr.address = parse_as_ipv4(addr_vec.as_slice())?.to_string();
        } else if let Nla::Address(addr_vec) = nla {
            peer = parse_as_ipv4(addr_vec.as_slice())?.to_string();
        } else if let Nla::CacheInfo(cache_info_vec) = nla {
            let cache_info = CacheInfo::parse(&CacheInfoBuffer::new(
                cache_info_vec.as_slice(),
            ))?;
            addr.preferred_lft = left_time_to_string(cache_info.ifa_preferred);
            addr.valid_lft = left_time_to_string(cache_info.ifa_valid);
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

    for nla in &nl_msg.nlas {
        if let Nla::Local(addr_vec) = nla {
            addr.address = parse_as_ipv6(addr_vec.as_slice())?.to_string();
            addr.peer_prefix_len = Some(addr.prefix_len);
            addr.prefix_len = 128;
        }
    }

    for nla in &nl_msg.nlas {
        if let Nla::Address(addr_vec) = nla {
            if addr.peer_prefix_len.is_some() {
                addr.peer = Some(parse_as_ipv6(addr_vec.as_slice())?);
            } else {
                addr.address = parse_as_ipv6(addr_vec.as_slice())?.to_string();
            }
        } else if let Nla::CacheInfo(cache_info_vec) = nla {
            let cache_info = CacheInfo::parse(&CacheInfoBuffer::new(
                cache_info_vec.as_slice(),
            ))?;
            addr.preferred_lft = left_time_to_string(cache_info.ifa_preferred);
            addr.valid_lft = left_time_to_string(cache_info.ifa_valid);
        } else if let Nla::Flags(flags) = nla {
            addr.flags = _Ipv6AddrFlags::from(*flags).0;
        }
    }

    Ok((iface_index, addr))
}

fn left_time_to_string(left_time: i32) -> String {
    if left_time == -1 {
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

#[derive(Clone, Eq, PartialEq, Debug)]
struct _Ipv6AddrFlags(Vec<Ipv6AddrFlag>);

impl From<u32> for _Ipv6AddrFlags {
    fn from(d: u32) -> Self {
        let mut got: u32 = 0;
        let mut ret = Vec::new();
        for flag in Ipv6AddrFlag::all() {
            if (d & (flag as u32)) > 0 {
                ret.push(flag);
                got += flag as u32;
            }
        }
        if got != d {
            eprintln!("Discarded unsupported IFA_FLAGS: {}", d - got);
        }
        Self(ret)
    }
}

impl From<&_Ipv6AddrFlags> for u32 {
    fn from(v: &_Ipv6AddrFlags) -> u32 {
        let mut d: u32 = 0;
        for flag in &v.0 {
            d += *flag as u32;
        }
        d
    }
}
