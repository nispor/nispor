// SPDX-License-Identifier: Apache-2.0

use std::{net::IpAddr, str::FromStr};

use rtnetlink::packet_route::{
    AddressFamily,
    address::{AddressAttribute, AddressMessage, CacheInfo},
};
use serde::{Deserialize, Serialize};

use super::super::query::is_ipv6_addr;
use crate::{
    AddressProtocol, AddressScope, Iface, IfaceConf, IpAddrFlag, IpFamily,
    Ipv4Info, Ipv6Info, NisporError,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct IpConf {
    pub addresses: Vec<IpAddrConf>,
}

impl From<&Ipv4Info> for IpConf {
    fn from(info: &Ipv4Info) -> Self {
        let mut addrs = Vec::new();
        for addr_info in &info.addresses {
            if addr_info.valid_lft == "forever" {
                addrs.push(IpAddrConf {
                    remove: false,
                    address: addr_info.address.clone(),
                    prefix_len: addr_info.prefix_len,
                    preferred_lft: addr_info.preferred_lft.clone(),
                    valid_lft: addr_info.valid_lft.clone(),
                    protocol: addr_info.protocol,
                    scope: Some(addr_info.scope),
                    flags: addr_info.flags.clone(),
                    label: None,
                    peer: addr_info.peer.clone(),
                    peer_prefix_len: None,
                });
            }
        }
        Self { addresses: addrs }
    }
}

impl From<&Ipv6Info> for IpConf {
    fn from(info: &Ipv6Info) -> Self {
        let mut addrs = Vec::new();
        for addr_info in &info.addresses {
            if addr_info.valid_lft == "forever" {
                addrs.push(IpAddrConf {
                    remove: false,
                    address: addr_info.address.clone(),
                    prefix_len: addr_info.prefix_len,
                    preferred_lft: addr_info.preferred_lft.clone(),
                    valid_lft: addr_info.valid_lft.clone(),
                    protocol: addr_info.protocol,
                    scope: Some(addr_info.scope),
                    flags: addr_info.flags.clone(),
                    label: None,
                    peer: addr_info.peer.map(|p| p.to_string()),
                    peer_prefix_len: addr_info.peer_prefix_len,
                });
            }
        }
        Self { addresses: addrs }
    }
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct IpAddrConf {
    #[serde(default)]
    pub remove: bool,
    pub address: String,
    pub prefix_len: u8,
    #[serde(default)]
    pub valid_lft: String,
    #[serde(default)]
    pub preferred_lft: String,
    pub protocol: Option<AddressProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<AddressScope>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flags: Vec<IpAddrFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_prefix_len: Option<u8>,
}

pub(crate) async fn change_ip_layer(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: &Iface,
) -> Result<(), NisporError> {
    if let Some(ip_conf) = des_iface.ipv4.as_ref() {
        apply_ip_conf(handle, cur_iface.index, ip_conf, IpFamily::Ipv4).await?;
    }
    if let Some(ip_conf) = des_iface.ipv6.as_ref() {
        apply_ip_conf(handle, cur_iface.index, ip_conf, IpFamily::Ipv6).await?;
    }

    Ok(())
}

async fn apply_ip_conf(
    handle: &rtnetlink::Handle,
    iface_index: u32,
    ip_conf: &IpConf,
    ip_family: IpFamily,
) -> Result<(), NisporError> {
    for addr_conf in &ip_conf.addresses {
        if addr_conf.remove {
            let mut nl_msg = AddressMessage::default();
            nl_msg.header.index = iface_index;
            nl_msg.header.prefix_len = addr_conf.prefix_len;
            nl_msg.header.family = match ip_family {
                IpFamily::Ipv4 => AddressFamily::Inet,
                IpFamily::Ipv6 => AddressFamily::Inet6,
            };
            nl_msg.attributes.push(AddressAttribute::Address(
                ip_addr_str_to_enum(&addr_conf.address)?,
            ));
            if let Some(protocol) = addr_conf.protocol {
                nl_msg
                    .attributes
                    .push(AddressAttribute::Protocol(protocol.into()));
            }
            if let Err(e) = handle.address().del(nl_msg).execute().await {
                if let rtnetlink::Error::NetlinkError(ref e) = e
                    && e.raw_code() == -libc::EADDRNOTAVAIL
                {
                    return Ok(());
                }
                return Err(e.into());
            }
        } else {
            let mut req = handle
                .address()
                .add(
                    iface_index,
                    ip_addr_str_to_enum(&addr_conf.address)?,
                    addr_conf.prefix_len,
                )
                .replace();

            if let Some(protocol) = addr_conf.protocol {
                req.message_mut()
                    .attributes
                    .push(AddressAttribute::Protocol(protocol.into()));
            }
            if let Some(scope) = addr_conf.scope {
                req.message_mut().header.scope = scope.into();
            }
            if !addr_conf.flags.is_empty() {
                let mut bits =
                    rtnetlink::packet_route::address::AddressFlags::empty();
                for flag in &addr_conf.flags {
                    bits |=
                        rtnetlink::packet_route::address::AddressFlags::from(
                            *flag,
                        );
                }
                req.message_mut()
                    .attributes
                    .push(AddressAttribute::Flags(bits));
            }
            if let Some(label) = &addr_conf.label {
                req.message_mut()
                    .attributes
                    .push(AddressAttribute::Label(label.clone()));
            }
            if let Some(peer) = &addr_conf.peer {
                let peer_addr = ip_addr_str_to_enum(peer)?;
                let prefix_len = addr_conf
                    .peer_prefix_len
                    .unwrap_or(addr_conf.prefix_len);
                // IFA_ADDRESS carries the peer; IFA_LOCAL carries the
                // local end.  Replace the builder-generated entries.
                let local =
                    ip_addr_str_to_enum(&addr_conf.address)?;
                let msg = req.message_mut();
                msg.header.prefix_len = prefix_len;
                msg.attributes
                    .retain(|a| {
                        !matches!(
                            a,
                            AddressAttribute::Address(_)
                                | AddressAttribute::Local(_)
                        )
                    });
                msg.attributes
                    .push(AddressAttribute::Local(local));
                msg.attributes
                    .push(AddressAttribute::Address(peer_addr));
            }

            if is_dynamic_ip(&addr_conf.preferred_lft, &addr_conf.valid_lft) {
                handle_dynamic_ip(
                    req.message_mut(),
                    &addr_conf.preferred_lft,
                    &addr_conf.valid_lft,
                )?;
            }
            req.execute().await?;
        }
    }
    Ok(())
}

fn ip_addr_str_to_enum(address: &str) -> Result<IpAddr, NisporError> {
    Ok(if is_ipv6_addr(address) {
        IpAddr::V6(std::net::Ipv6Addr::from_str(address)?)
    } else {
        IpAddr::V4(std::net::Ipv4Addr::from_str(address)?)
    })
}

fn is_dynamic_ip(preferred_lft: &str, valid_lft: &str) -> bool {
    (preferred_lft != "forever" && !preferred_lft.is_empty())
        || (valid_lft != "forever" && !valid_lft.is_empty())
}

fn gen_cache_info(
    preferred_lft: &str,
    valid_lft: &str,
) -> Result<CacheInfo, NisporError> {
    let mut ret = CacheInfo::default();
    ret.ifa_preferred = parse_lft_sec("preferred_lft", preferred_lft)?;
    ret.ifa_valid = parse_lft_sec("valid_lft", valid_lft)?;
    Ok(ret)
}

fn handle_dynamic_ip(
    nl_msg: &mut AddressMessage,
    preferred_lft: &str,
    valid_lft: &str,
) -> Result<(), NisporError> {
    nl_msg
        .attributes
        .push(AddressAttribute::CacheInfo(gen_cache_info(
            preferred_lft,
            valid_lft,
        )?));
    Ok(())
}

fn parse_lft_sec(name: &str, lft_str: &str) -> Result<u32, NisporError> {
    let e = NisporError::invalid_argument(format!(
        "Invalid {name} format: expect format 50sec, got {lft_str}"
    ));
    match lft_str.strip_suffix("sec") {
        Some(a) => a.parse().map_err(|_| {
            log::error!("{e}");
            e
        }),
        None => {
            log::error!("{e}");
            Err(e)
        }
    }
}
