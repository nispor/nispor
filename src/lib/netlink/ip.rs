use crate::ifaces::get_iface_name_by_index;
use crate::Iface;
use crate::Ipv4AddrInfo;
use crate::Ipv4Info;
use crate::Ipv6AddrInfo;
use crate::Ipv6Info;
use netlink_packet_route::rtnl::address::nlas::Nla::{
    Address, CacheInfo, Local,
};
use netlink_packet_route::rtnl::AddressMessage;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::net::Ipv6Addr;

const AF_INET: u8 = 2;
const AF_INET6: u8 = 10;

pub(crate) fn fill_ip_addr(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &AddressMessage,
) {
    match nl_msg.header.family {
        AF_INET => {
            let (iface_index, addr) = parse_ipv4_nlas(nl_msg);
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
                iface_states.get_mut(&iface_name).unwrap().ipv4 =
                    Some(new_ip4_info);
            }
        }
        AF_INET6 => {
            let (iface_index, addr) = parse_ipv6_nlas(nl_msg);
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
                iface_states.get_mut(&iface_name).unwrap().ipv6 =
                    Some(new_ip6_info);
            }
        }
        _ => {
            println!(
                "unknown address family {} {:?}",
                nl_msg.header.family, nl_msg
            );
        }
    };
}

// TODO: remove the dupcode between parse_ipv4_nlas() and parse_ipv6_nlas()
fn parse_ipv4_nlas(nl_msg: &AddressMessage) -> (u32, Ipv4AddrInfo) {
    let iface_index = nl_msg.header.index;
    let mut addr: Ipv4AddrInfo = Default::default();
    addr.prefix_len = nl_msg.header.prefix_len;
    let mut peer = String::new();
    for nla in &nl_msg.nlas {
        if let Local(addr_vec) = nla {
            addr.address = Ipv4Addr::from([
                addr_vec[0],
                addr_vec[1],
                addr_vec[2],
                addr_vec[3],
            ])
            .to_string();
        } else if let Address(addr_vec) = nla {
            peer = Ipv4Addr::from([
                addr_vec[0],
                addr_vec[1],
                addr_vec[2],
                addr_vec[3],
            ])
            .to_string();
        } else if let CacheInfo(cache_info_vec) = nla {
            let cache_info = parse_cache_info(&cache_info_vec);
            addr.preferred_lft = left_time_to_string(cache_info.ifa_prefered);
            addr.valid_lft = left_time_to_string(cache_info.ifa_valid);
        }
    }

    if peer != addr.address {
        addr.peer = Some(peer)
    }

    (iface_index, addr)
}

fn parse_ipv6_nlas(nl_msg: &AddressMessage) -> (u32, Ipv6AddrInfo) {
    let iface_index = nl_msg.header.index;
    let mut addr: Ipv6AddrInfo = Default::default();
    addr.prefix_len = nl_msg.header.prefix_len;

    for nla in &nl_msg.nlas {
        if let Address(addr_vec) = nla {
            if addr_vec.len() == 16 {
                let mut addr_bytes = [0u8; 16];
                addr_bytes.copy_from_slice(&addr_vec[..16]);
                addr.address = Ipv6Addr::from(addr_bytes).to_string();
            } else {
                panic!("Got invalid IPv6 address u8, the length is not 16 ");
            }
        } else if let CacheInfo(cache_info_vec) = nla {
            let cache_info = parse_cache_info(&cache_info_vec);
            addr.preferred_lft = left_time_to_string(cache_info.ifa_prefered);
            addr.valid_lft = left_time_to_string(cache_info.ifa_valid);
        }
    }

    (iface_index, addr)
}

struct IfaCacheInfo {
    ifa_prefered: u32,
    ifa_valid: u32,
    //cstamp: u32,
    //tstamp: u32,
}

fn parse_cache_info(cache_info_raw: &[u8]) -> IfaCacheInfo {
    if cache_info_raw.len() != 16 {
        panic!(
            "Got invalid ifa_cacheinfo, expect [u8; 32], got {} u8",
            cache_info_raw.len()
        );
    } else {
        // The struct ifa_cacheinfo is storing valid time as second u32
        IfaCacheInfo {
            ifa_prefered: u32::from_ne_bytes([
                cache_info_raw[0],
                cache_info_raw[1],
                cache_info_raw[2],
                cache_info_raw[3],
            ]),
            ifa_valid: u32::from_ne_bytes([
                cache_info_raw[4],
                cache_info_raw[5],
                cache_info_raw[6],
                cache_info_raw[7],
            ]),
        }
    }
}

fn left_time_to_string(left_time: u32) -> String {
    if left_time == u32::MAX {
        "forever".into()
    } else {
        format!("{}sec", left_time)
    }
}
