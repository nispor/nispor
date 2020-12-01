use crate::ifaces::get_iface_name_by_index;
use crate::netlink::nla::parse_as_ipv4;
use crate::netlink::nla::parse_as_ipv6;
use crate::Iface;
use crate::Ipv4AddrInfo;
use crate::Ipv4Info;
use crate::Ipv6AddrInfo;
use crate::Ipv6Info;
use crate::NisporError;
use netlink_packet_route::rtnl::address::nlas::Nla::{
    Address, CacheInfo, Local,
};
use netlink_packet_route::rtnl::AddressMessage;
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
            let iface_name =
                get_iface_name_by_index(&iface_states, iface_index);
            if iface_name != "" {
                let new_ip4_info = match &iface_states[&iface_name].ipv4 {
                    Some(ip_info) => {
                        let mut new_ip_info = ip_info.clone();
                        new_ip_info.addresses.push(addr);
                        new_ip_info
                    }
                    None => Ipv4Info {
                        addresses: vec![addr],
                    },
                };
                if let Some(iface) = iface_states.get_mut(&iface_name) {
                    iface.ipv4 = Some(new_ip4_info);
                }
            }
        }
        AF_INET6 => {
            let (iface_index, addr) = parse_ipv6_nlas(nl_msg)?;
            let iface_name =
                get_iface_name_by_index(&iface_states, iface_index);
            if iface_name != "" {
                let new_ip6_info = match &iface_states[&iface_name].ipv6 {
                    Some(ip_info) => {
                        let mut new_ip_info = ip_info.clone();
                        new_ip_info.addresses.push(addr);
                        new_ip_info
                    }
                    None => Ipv6Info {
                        addresses: vec![addr],
                    },
                };
                if let Some(iface) = iface_states.get_mut(&iface_name) {
                    iface.ipv6 = Some(new_ip6_info);
                }
            }
        }
        _ => {
            println!(
                "unknown address family {} {:?}",
                nl_msg.header.family, nl_msg
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
    let mut addr: Ipv4AddrInfo = Default::default();
    addr.prefix_len = nl_msg.header.prefix_len;
    let mut peer = String::new();
    for nla in &nl_msg.nlas {
        if let Local(addr_vec) = nla {
            addr.address = parse_as_ipv4(addr_vec.as_slice()).to_string();
        } else if let Address(addr_vec) = nla {
            peer = parse_as_ipv4(addr_vec.as_slice()).to_string();
        } else if let CacheInfo(cache_info_vec) = nla {
            let cache_info = parse_cache_info(&cache_info_vec)?;
            addr.preferred_lft = left_time_to_string(cache_info.ifa_prefered);
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
    let mut addr: Ipv6AddrInfo = Default::default();
    addr.prefix_len = nl_msg.header.prefix_len;

    for nla in &nl_msg.nlas {
        if let Address(addr_vec) = nla {
            addr.address = parse_as_ipv6(addr_vec.as_slice()).to_string();
        } else if let CacheInfo(cache_info_vec) = nla {
            let cache_info = parse_cache_info(&cache_info_vec)?;
            addr.preferred_lft = left_time_to_string(cache_info.ifa_prefered);
            addr.valid_lft = left_time_to_string(cache_info.ifa_valid);
        }
    }

    Ok((iface_index, addr))
}

struct IfaCacheInfo {
    ifa_prefered: u32,
    ifa_valid: u32,
    /*cstamp: u32,
    tstamp: u32, */
}

fn parse_cache_info(
    cache_info_raw: &[u8],
) -> Result<IfaCacheInfo, NisporError> {
    if cache_info_raw.len() != 16 {
        panic!(
            "Got invalid ifa_cacheinfo, expect [u8; 32], got {} u8",
            cache_info_raw.len()
        );
    } else {
        // The struct ifa_cacheinfo is storing valid time as second u32
        let err_msg = "wrong index at cache_info_raw parsing";
        Ok(IfaCacheInfo {
            ifa_prefered: u32::from_ne_bytes([
                *cache_info_raw.get(0).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(1).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(2).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(3).ok_or(NisporError::bug(err_msg))?,
            ]),
            ifa_valid: u32::from_ne_bytes([
                *cache_info_raw.get(4).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(5).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(6).ok_or(NisporError::bug(err_msg))?,
                *cache_info_raw.get(7).ok_or(NisporError::bug(err_msg))?,
            ]),
        })
    }
}

fn left_time_to_string(left_time: u32) -> String {
    if left_time == std::u32::MAX {
        "forever".into()
    } else {
        format!("{}sec", left_time)
    }
}
