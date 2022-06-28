use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::Read;
use std::net::IpAddr;

use futures::stream::TryStreamExt;
use mptcp_pm::{
    MptcpPathManagerAddressAttr, MptcpPathManagerAddressAttrFlag,
    MptcpPathManagerAttr, MptcpPathManagerLimitsAttr, MptcpPathManagerMessage,
};
use serde::{Deserialize, Serialize};

use crate::{Iface, NisporError};

const MPTCP_SYSCTL_PATH: &str = "/proc/sys/net/mptcp/enabled";

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Mptcp {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_addr_accepted_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subflows_limit: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<MptcpAddress>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum MptcpAddressFlag {
    Signal,
    Subflow,
    Backup,
    Fullmesh,
    Implicit,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub struct MptcpAddress {
    pub address: IpAddr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<MptcpAddressFlag>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iface: Option<String>,
    #[serde(skip)]
    pub iface_index: Option<i32>,
}

pub(crate) async fn get_mptcp() -> Result<Mptcp, NisporError> {
    let mut ret = Mptcp {
        enabled: is_mptcp_enabled(),
        ..Default::default()
    };
    if !ret.enabled {
        return Ok(ret);
    }

    let (connection, handle, _) = mptcp_pm::new_connection()?;
    tokio::spawn(connection);

    let mut limits_handle = handle.limits().get().execute().await;

    while let Some(genl_msg) = limits_handle.try_next().await? {
        let mptcp_msg = genl_msg.payload;
        for nla in &mptcp_msg.nlas {
            match nla {
                MptcpPathManagerAttr::Limits(
                    MptcpPathManagerLimitsAttr::RcvAddAddrs(d),
                ) => {
                    ret.add_addr_accepted_limit = Some(*d);
                }
                MptcpPathManagerAttr::Limits(
                    MptcpPathManagerLimitsAttr::Subflows(d),
                ) => {
                    ret.subflows_limit = Some(*d);
                }
                _ => {
                    log::info!("Unsupported MPTCP netlink attribute {:?}", nla)
                }
            }
        }
    }

    let mut address_handle = handle.address().get().execute().await;
    let mut addresses = Vec::new();
    while let Some(genl_msg) = address_handle.try_next().await? {
        if let Some(addr) = mptcp_msg_to_nispor(&genl_msg.payload) {
            addresses.push(addr);
        }
    }
    ret.addresses = Some(addresses);

    Ok(ret)
}

fn is_mptcp_enabled() -> bool {
    if let Ok(mut fd) = std::fs::File::open(MPTCP_SYSCTL_PATH) {
        let mut content = [0u8; 1];
        if fd.read_exact(&mut content).is_err() {
            false
        } else {
            content[0] == b'1'
        }
    } else {
        false
    }
}

// * Place address with interface to Iface
// * Replace index to interface name
pub(crate) fn merge_mptcp_info(
    iface_states: &mut HashMap<String, Iface>,
    mptcp: &mut Mptcp,
) {
    let mut addr_index: HashMap<String, Vec<MptcpAddress>> = HashMap::new();
    let mut iface_index_map: HashMap<u32, String> = HashMap::new();

    for iface in iface_states.values() {
        iface_index_map.insert(iface.index, iface.name.clone());
    }

    if let Some(addrs) = mptcp.addresses.as_mut() {
        for addr in addrs {
            if let Some(iface_index) = addr.iface_index.as_ref() {
                if let Ok(i) = u32::try_from(*iface_index) {
                    if let Some(iface_name) = iface_index_map.get(&i) {
                        addr.iface = Some(iface_name.to_string());
                        match addr_index.entry(iface_name.to_string()) {
                            Entry::Occupied(o) => {
                                o.into_mut().push(addr.clone());
                            }
                            Entry::Vacant(v) => {
                                v.insert(vec![addr.clone()]);
                            }
                        };
                    } else {
                        addr.iface = Some(iface_index.to_string());
                    }
                } else {
                    log::error!("BUG: Got invalid iface index in  {:?}", addr);
                }
            }
        }
    }

    for iface in iface_states.values_mut() {
        if let Some(addrs) = addr_index.remove(iface.name.as_str()) {
            iface.mptcp = Some(addrs);
        }
    }
}

fn mptcp_msg_to_nispor(
    mptcp_msg: &MptcpPathManagerMessage,
) -> Option<MptcpAddress> {
    let mut address = None;
    for nla in &mptcp_msg.nlas {
        if let MptcpPathManagerAttr::Address(
            MptcpPathManagerAddressAttr::Addr4(ip),
        ) = nla
        {
            address = Some(IpAddr::V4(*ip));
            break;
        } else if let MptcpPathManagerAttr::Address(
            MptcpPathManagerAddressAttr::Addr6(ip),
        ) = nla
        {
            address = Some(IpAddr::V6(*ip));
            break;
        }
    }
    if let Some(address) = address {
        let mut ret = MptcpAddress {
            address,
            id: None,
            port: None,
            flags: None,
            iface: None,
            iface_index: None,
        };
        for nla in &mptcp_msg.nlas {
            if let MptcpPathManagerAttr::Address(addr_attr) = nla {
                match addr_attr {
                    MptcpPathManagerAddressAttr::Flags(flags) => {
                        ret.flags = Some(mptcp_flags_to_nispor(flags));
                    }
                    MptcpPathManagerAddressAttr::IfIndex(i) => {
                        ret.iface_index = Some(*i);
                    }
                    MptcpPathManagerAddressAttr::Port(i) => {
                        if *i != 0 {
                            ret.port = Some(*i);
                        }
                    }
                    MptcpPathManagerAddressAttr::Id(i) => {
                        ret.id = Some(*i);
                    }
                    _ => (),
                }
            }
        }
        Some(ret)
    } else {
        None
    }
}

fn mptcp_flags_to_nispor(
    flags: &[MptcpPathManagerAddressAttrFlag],
) -> Vec<MptcpAddressFlag> {
    let mut ret = Vec::new();
    for flag in flags {
        if let Some(f) = match flag {
            MptcpPathManagerAddressAttrFlag::Signal => {
                Some(MptcpAddressFlag::Signal)
            }
            MptcpPathManagerAddressAttrFlag::Subflow => {
                Some(MptcpAddressFlag::Subflow)
            }
            MptcpPathManagerAddressAttrFlag::Backup => {
                Some(MptcpAddressFlag::Backup)
            }
            MptcpPathManagerAddressAttrFlag::Fullmesh => {
                Some(MptcpAddressFlag::Fullmesh)
            }
            MptcpPathManagerAddressAttrFlag::Implicit => {
                Some(MptcpAddressFlag::Implicit)
            }
            _ => {
                log::info!("Unsupported address flag {:?}", flag);
                None
            }
        } {
            ret.push(f);
        }
    }
    ret
}
