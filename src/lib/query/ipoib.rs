// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::packet_route::link::{self, InfoData, InfoIpoib};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType};

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum IpoibMode {
    /// using unreliable datagram QPs
    #[default]
    Datagram,
    /// using connected QPs
    Connected,
    Other(u16),
}

impl From<link::IpoibMode> for IpoibMode {
    fn from(d: link::IpoibMode) -> Self {
        match d {
            link::IpoibMode::Datagram => Self::Datagram,
            link::IpoibMode::Connected => Self::Connected,
            _ => Self::Other(d.into()),
        }
    }
}

impl From<IpoibMode> for link::IpoibMode {
    fn from(v: IpoibMode) -> Self {
        match v {
            IpoibMode::Datagram => Self::Datagram,
            IpoibMode::Connected => Self::Connected,
            IpoibMode::Other(d) => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct IpoibInfo {
    pub pkey: u16,
    pub mode: IpoibMode,
    pub umcast: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_iface: Option<String>,
}

pub(crate) fn get_ipoib_info(data: &InfoData) -> Option<IpoibInfo> {
    if let InfoData::Ipoib(infos) = data {
        let mut ipoib_info = IpoibInfo::default();
        for info in infos {
            if let InfoIpoib::Pkey(d) = *info {
                ipoib_info.pkey = d;
            } else if let InfoIpoib::Mode(d) = *info {
                ipoib_info.mode = d.into();
            } else if let InfoIpoib::UmCast(d) = *info {
                ipoib_info.umcast = d > 0;
            } else {
                log::debug!("Unknown IPoIB info {info:?}")
            }
        }
        Some(ipoib_info)
    } else {
        None
    }
}

pub(crate) fn ipoib_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    convert_base_iface_index_to_name(iface_states);
}

fn convert_base_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Ipoib {
            continue;
        }
        if let Some(ref mut ib_info) = iface.ipoib
            && let Some(base_iface_name) = &ib_info
                .base_iface
                .as_ref()
                .and_then(|i| index_to_name.get(i))
        {
            ib_info.base_iface = Some(base_iface_name.to_string());
        }
    }
}
