use crate::netlink::parse_as_u32;
use crate::netlink::parse_as_u8;
use crate::NisporError;
use netlink_packet_route::rtnl::link::nlas::InfoData;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use serde_derive::{Deserialize, Serialize};

const IFF_TUN: u8 = 1;
const IFF_TAP: u8 = 2;

const IFLA_TUN_OWNER: u16 = 1;
const IFLA_TUN_GROUP: u16 = 2;
const IFLA_TUN_TYPE: u16 = 3;
const IFLA_TUN_PI: u16 = 4;
const IFLA_TUN_VNET_HDR: u16 = 5;
const IFLA_TUN_PERSIST: u16 = 6;
const IFLA_TUN_MULTI_QUEUE: u16 = 7;
const IFLA_TUN_NUM_QUEUES: u16 = 8;
const IFLA_TUN_NUM_DISABLED_QUEUES: u16 = 9;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct TunInfo {
    pub mode: TunMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<u32>,
    pub pi: bool,
    pub vnet_hdr: bool,
    pub multi_queue: bool,
    pub persist: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_queues: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_disabled_queues: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum TunMode {
    Tun,
    Tap,
    Unknown,
}

impl Default for TunMode {
    fn default() -> Self {
        TunMode::Unknown
    }
}

impl From<u8> for TunMode {
    fn from(d: u8) -> Self {
        match d {
            IFF_TUN => TunMode::Tun,
            IFF_TAP => TunMode::Tap,
            _ => {
                eprintln!("Unhandled TUN mode {}", d);
                TunMode::Unknown
            }
        }
    }
}

pub(crate) fn get_tun_info(data: &InfoData) -> Result<TunInfo, NisporError> {
    let mut tun_info = TunInfo::default();
    if let InfoData::Tun(raw) = data {
        let nlas = NlasIterator::new(raw);
        for nla in nlas {
            let nla = nla?;
            match nla.kind() {
                IFLA_TUN_OWNER => {
                    tun_info.owner = Some(parse_as_u32(nla.value()));
                }
                IFLA_TUN_GROUP => {
                    tun_info.group = Some(parse_as_u32(nla.value()));
                }
                IFLA_TUN_TYPE => {
                    tun_info.mode = parse_as_u8(nla.value()).into();
                }
                IFLA_TUN_PI => {
                    tun_info.pi = parse_as_u8(nla.value()) > 0;
                }
                IFLA_TUN_VNET_HDR => {
                    tun_info.vnet_hdr = parse_as_u8(nla.value()) > 0;
                }
                IFLA_TUN_PERSIST => {
                    tun_info.persist = parse_as_u8(nla.value()) > 0;
                }
                IFLA_TUN_MULTI_QUEUE => {
                    tun_info.multi_queue = parse_as_u8(nla.value()) > 0;
                }
                IFLA_TUN_NUM_QUEUES => {
                    tun_info.num_queues = Some(parse_as_u32(nla.value()));
                }
                IFLA_TUN_NUM_DISABLED_QUEUES => {
                    tun_info.num_disabled_queues =
                        Some(parse_as_u32(nla.value()));
                }
                _ => {
                    eprintln!(
                        "Unhandled TUN NLA {} {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            }
        }
    }
    Ok(tun_info)
}
