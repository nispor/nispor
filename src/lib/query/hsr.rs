// SPDX-License-Identifier: Apache-2.0
use std::collections::HashMap;

use netlink_packet_route::link::{InfoData, InfoHsr};
use serde::{Deserialize, Serialize};

use crate::mac::{parse_as_mac, ETH_ALEN};
use crate::{Iface, IfaceType};

const HSR_PROTOCOL_HSR: u8 = 0;
const HSR_PROTOCOL_PRP: u8 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum HsrProtocol {
    Hsr,
    Prp,
    Other(u8),
    Unknown,
}

impl Default for HsrProtocol {
    fn default() -> Self {
        HsrProtocol::Unknown
    }
}

impl From<u8> for HsrProtocol {
    fn from(d: u8) -> Self {
        match d {
            HSR_PROTOCOL_HSR => Self::Hsr,
            HSR_PROTOCOL_PRP => Self::Prp,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct HsrInfo {
    pub port1: Option<String>,
    pub port2: Option<String>,
    pub supervision_addr: String,
    pub seq_nr: u16,
    pub multicast_spec: u8,
    pub version: u8,
    pub protocol: HsrProtocol,
    #[serde(skip_serializing)]
    _port1_ifindex: u32,
    #[serde(skip_serializing)]
    _port2_ifindex: u32,
}

pub(crate) fn get_hsr_info(data: &InfoData) -> Option<HsrInfo> {
    if let InfoData::Hsr(infos) = data {
        let mut hsr_info = HsrInfo::default();
        for info in infos {
            match *info {
                InfoHsr::Port1(d) => {
                    hsr_info._port1_ifindex = d;
                }
                InfoHsr::Port2(d) => {
                    hsr_info._port2_ifindex = d;
                }
                InfoHsr::SupervisionAddr(d) => {
                    hsr_info.supervision_addr =
                        parse_as_mac(ETH_ALEN, &d).unwrap_or_default();
                }
                InfoHsr::SeqNr(d) => {
                    hsr_info.seq_nr = d;
                }
                InfoHsr::MulticastSpec(d) => {
                    hsr_info.multicast_spec = d;
                }
                InfoHsr::Version(d) => {
                    hsr_info.version = d;
                }
                InfoHsr::Protocol(d) => {
                    hsr_info.protocol = u8::from(d).into();
                }
                _ => {
                    log::warn!("Unknown HSR info {:?}", info);
                }
            }
        }
        Some(hsr_info)
    } else {
        None
    }
}

pub(crate) fn hsr_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    fill_port_iface_names(iface_states);
}

fn fill_port_iface_names(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(iface.index, iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Hsr {
            continue;
        }
        if let Some(ref mut hsr_info) = iface.hsr {
            if let Some(port1_iface_name) =
                index_to_name.get(&hsr_info._port1_ifindex)
            {
                hsr_info.port1 = Some(port1_iface_name.to_string());
            }
            if let Some(port2_iface_name) =
                index_to_name.get(&hsr_info._port2_ifindex)
            {
                hsr_info.port2 = Some(port2_iface_name.to_string());
            }
        }
    }
}
