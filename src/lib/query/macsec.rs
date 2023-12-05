// SPDX-License-Identifier: Apache-2.0
use std::collections::HashMap;

use netlink_packet_route::link::{InfoData, InfoMacSec};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

const MACSEC_VALIDATE_DISABLED: u8 = 0;
const MACSEC_VALIDATE_CHECK: u8 = 1;
const MACSEC_VALIDATE_STRICT: u8 = 2;

const MACSEC_OFFLOAD_OFF: u8 = 0;
const MACSEC_OFFLOAD_PHY: u8 = 1;
const MACSEC_OFFLOAD_MAC: u8 = 2;

const MACSEC_CIPHER_ID_GCM_AES_128: u64 = 0x0080C20001000001;
const MACSEC_CIPHER_ID_GCM_AES_256: u64 = 0x0080C20001000002;
const MACSEC_CIPHER_ID_GCM_AES_XPN_128: u64 = 0x0080C20001000003;
const MACSEC_CIPHER_ID_GCM_AES_XPN_256: u64 = 0x0080C20001000004;
const MACSEC_DEFAULT_CIPHER_ID: u64 = 0x0080020001000001;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MacSecValidate {
    Disabled,
    Check,
    Strict,
    Other(u8),
    Unknown,
}

impl Default for MacSecValidate {
    fn default() -> Self {
        MacSecValidate::Unknown
    }
}

impl From<u8> for MacSecValidate {
    fn from(d: u8) -> Self {
        match d {
            MACSEC_VALIDATE_DISABLED => Self::Disabled,
            MACSEC_VALIDATE_CHECK => Self::Check,
            MACSEC_VALIDATE_STRICT => Self::Strict,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum MacSecOffload {
    Off,
    Phy,
    Mac,
    Other(u8),
    Unknown,
}

impl Default for MacSecOffload {
    fn default() -> Self {
        MacSecOffload::Unknown
    }
}

impl From<u8> for MacSecOffload {
    fn from(d: u8) -> Self {
        match d {
            MACSEC_OFFLOAD_OFF => Self::Off,
            MACSEC_OFFLOAD_PHY => Self::Phy,
            MACSEC_OFFLOAD_MAC => Self::Mac,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
#[non_exhaustive]
pub enum MacSecCipherId {
    GcmAes128,
    GcmAes256,
    GcmAesXpn128,
    GcmAesXpn256,
    Other(u64),
}

impl Default for MacSecCipherId {
    fn default() -> Self {
        MacSecCipherId::GcmAes128
    }
}

impl From<u64> for MacSecCipherId {
    fn from(d: u64) -> Self {
        match d {
            MACSEC_DEFAULT_CIPHER_ID => Self::GcmAes128,
            MACSEC_CIPHER_ID_GCM_AES_128 => Self::GcmAes128,
            MACSEC_CIPHER_ID_GCM_AES_256 => Self::GcmAes256,
            MACSEC_CIPHER_ID_GCM_AES_XPN_128 => Self::GcmAesXpn128,
            MACSEC_CIPHER_ID_GCM_AES_XPN_256 => Self::GcmAesXpn256,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct MacSecInfo {
    pub sci: u64,
    pub port: u16,
    pub icv_len: u8,
    pub cipher: MacSecCipherId,
    pub window: u32,
    pub encoding_sa: u8,
    pub encrypt: bool,
    pub protect: bool,
    pub send_sci: bool,
    pub end_station: bool,
    pub scb: bool,
    pub replay_protect: bool,
    pub validate: MacSecValidate,
    pub offload: MacSecOffload,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_iface: Option<String>,
}

pub(crate) fn get_macsec_info(data: &InfoData) -> Option<MacSecInfo> {
    if let InfoData::MacSec(infos) = data {
        let mut macsec_info = MacSecInfo::default();
        for info in infos {
            match *info {
                InfoMacSec::Sci(d) => {
                    macsec_info.sci = d;
                }
                InfoMacSec::Port(d) => {
                    macsec_info.port = d;
                }
                InfoMacSec::IcvLen(d) => {
                    macsec_info.icv_len = d;
                }
                InfoMacSec::CipherSuite(d) => {
                    macsec_info.cipher = u64::from(d).into();
                }
                InfoMacSec::Window(d) => {
                    macsec_info.window = d;
                }
                InfoMacSec::EncodingSa(d) => {
                    macsec_info.encoding_sa = d;
                }
                InfoMacSec::Encrypt(d) => {
                    macsec_info.encrypt = d > 0;
                }
                InfoMacSec::Protect(d) => {
                    macsec_info.protect = d > 0;
                }
                InfoMacSec::IncSci(d) => {
                    macsec_info.send_sci = d > 0;
                }
                InfoMacSec::Es(d) => {
                    macsec_info.end_station = d > 0;
                }
                InfoMacSec::Scb(d) => {
                    macsec_info.scb = d > 0;
                }
                InfoMacSec::ReplayProtect(d) => {
                    macsec_info.replay_protect = d > 0;
                }
                InfoMacSec::Validation(d) => {
                    macsec_info.validate = u8::from(d).into();
                }
                InfoMacSec::Offload(d) => {
                    macsec_info.offload = u8::from(d).into();
                }
                _ => {
                    log::debug!("Unknown MACsec info {:?}", info)
                }
            }
        }
        Some(macsec_info)
    } else {
        None
    }
}

pub(crate) fn macsec_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::MacSec {
            continue;
        }
        if let Some(ref mut macsec_info) = iface.macsec {
            if let Some(base_iface_name) = &macsec_info
                .base_iface
                .as_ref()
                .and_then(|i| index_to_name.get(i))
            {
                macsec_info.base_iface = Some(base_iface_name.to_string());
            }
        }
    }
}
