use crate::netlink::{
    get_ip_addr, get_ip_prefix_len, parse_apply_netlink_error,
};
use crate::Iface;
use crate::NisporError;
use futures::stream::TryStreamExt;
use netlink_packet_route;
use netlink_packet_route::rtnl::AddressMessage;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::net::IpAddr;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv4Info {
    pub addresses: Vec<Ipv4AddrInfo>,
}
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv6Info {
    pub addresses: Vec<Ipv6AddrInfo>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Ipv6AddrInfo {
    pub address: String,
    pub prefix_len: u8,
    // The renaming seonds for this address be valid
    pub valid_lft: String,
    // The renaming seonds for this address be preferred
    pub preferred_lft: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct IpConf {
    pub addresses: Vec<IpAddrConf>,
}

pub enum IpFamily {
    Ipv4,
    Ipv6,
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Default,
)]
pub struct IpAddrConf {
    pub address: String,
    pub prefix_len: u8,
}

impl IpConf {
    pub async fn apply(
        &self,
        handle: &rtnetlink::Handle,
        cur_iface: &Iface,
        family: IpFamily,
    ) -> Result<(), NisporError> {
        let has_ipv6_link_local_in_desire = self.addresses.iter().any(|addr| {
            is_ipv6_unicast_link_local(&addr.address, addr.prefix_len)
        });
        let mut cur_ip_addr_confs = HashSet::new();
        let mut des_ip_addr_confs = HashSet::new();
        let mut nl_addr_msgs = get_nl_addr_msgs(handle).await?;

        for des_addr in &self.addresses {
            des_ip_addr_confs.insert(IpAddrConf {
                address: des_addr.address.clone(),
                prefix_len: des_addr.prefix_len,
            });
        }
        match family {
            IpFamily::Ipv4 => {
                if let Some(Ipv4Info {
                    addresses: cur_addresses,
                }) = &cur_iface.ipv4
                {
                    for cur_addr in cur_addresses {
                        cur_ip_addr_confs.insert(IpAddrConf {
                            address: cur_addr.address.clone(),
                            prefix_len: cur_addr.prefix_len,
                        });
                    }
                }
            }
            IpFamily::Ipv6 => {
                if let Some(Ipv6Info {
                    addresses: cur_addresses,
                }) = &cur_iface.ipv6
                {
                    for cur_addr in cur_addresses {
                        cur_ip_addr_confs.insert(IpAddrConf {
                            address: cur_addr.address.clone(),
                            prefix_len: cur_addr.prefix_len,
                        });
                    }
                }
            }
        };

        for addr_to_remove in &cur_ip_addr_confs - &des_ip_addr_confs {
            // Don't remove link local address unless desire state has
            // link local address
            if has_ipv6_link_local_in_desire
                || !is_ipv6_unicast_link_local(
                    &addr_to_remove.address,
                    addr_to_remove.prefix_len,
                )
            {
                if let Some(nl_addr_msg) = nl_addr_msgs.remove(&format!(
                    "{}/{}",
                    &addr_to_remove.address, addr_to_remove.prefix_len
                )) {
                    remove_ip_addr_conf(handle, &addr_to_remove, nl_addr_msg)
                        .await?;
                }
            }
        }

        for addr_to_add in &des_ip_addr_confs - &cur_ip_addr_confs {
            add_ip_addr_conf(handle, cur_iface.index, &addr_to_add).await?;
        }
        Ok(())
    }
}

// TODO: Rust offical has std::net::Ipv6Addr::is_unicast_link_local() in
// experimental.
fn is_ipv6_unicast_link_local(ip: &str, prefix: u8) -> bool {
    // The unicast link local address range is fe80::/10.
    is_ipv6_addr(ip)
        && ip.len() >= 3
        && ["fe8", "fe9", "fea", "feb"].contains(&&ip[..3])
        && prefix >= 10
}

async fn remove_ip_addr_conf(
    handle: &rtnetlink::Handle,
    ip_addr_conf: &IpAddrConf,
    nl_addr_msg: AddressMessage,
) -> Result<(), NisporError> {
    if let Err(rtnetlink::Error::NetlinkError(e)) =
        handle.address().del(nl_addr_msg).execute().await
    {
        eprintln!(
            "Failed to remove IP address {}/{}: {}",
            ip_addr_conf.address, ip_addr_conf.prefix_len, &e
        );
        Err(parse_apply_netlink_error(&e))
    } else {
        Ok(())
    }
}

fn is_ipv6_addr(addr: &str) -> bool {
    addr.contains(":")
}

async fn add_ip_addr_conf(
    handle: &rtnetlink::Handle,
    iface_index: u32,
    ip_addr_conf: &IpAddrConf,
) -> Result<(), NisporError> {
    let ip_addr = if is_ipv6_addr(&ip_addr_conf.address) {
        IpAddr::V6(std::net::Ipv6Addr::from_str(&ip_addr_conf.address)?)
    } else {
        IpAddr::V4(std::net::Ipv4Addr::from_str(&ip_addr_conf.address)?)
    };
    if let Err(rtnetlink::Error::NetlinkError(e)) = handle
        .address()
        .add(iface_index, ip_addr, ip_addr_conf.prefix_len)
        .execute()
        .await
    {
        Err(parse_apply_netlink_error(&e))
    } else {
        Ok(())
    }
}

async fn get_nl_addr_msgs(
    handle: &rtnetlink::Handle,
) -> Result<HashMap<String, AddressMessage>, NisporError> {
    let mut msgs = HashMap::new();
    let mut addrs = handle.address().get().execute();
    while let Some(nl_addr_msg) = addrs.try_next().await? {
        let full_address = format!(
            "{}/{}",
            get_ip_addr(&nl_addr_msg),
            get_ip_prefix_len(&nl_addr_msg)
        );
        msgs.insert(full_address, nl_addr_msg);
    }

    Ok(msgs)
}
