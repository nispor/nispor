// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::net::IpAddr;

use crate::{
    Iface, Ipv4AddrInfo, Ipv4Info, Ipv6AddrFlag, Ipv6AddrInfo, Ipv6Info,
    NisporError,
};
use netlink_packet_route::address::{AddressAttribute, AddressMessage};

pub(crate) fn fill_ip_addr(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &AddressMessage,
) -> Result<(), NisporError> {
    match nl_msg.header.family {
        netlink_packet_route::AddressFamily::Inet => {
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
        netlink_packet_route::AddressFamily::Inet6 => {
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
            addr.flags = flags
                .as_slice()
                .iter()
                .map(|f| Ipv6AddrFlag::from(*f))
                .collect();
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
