// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use wl_nl80211::{
    Nl80211Attr, Nl80211BssInfo, Nl80211Element, Nl80211Handle,
    Nl80211InterfaceType, Nl80211RateInfo, Nl80211StationFlags,
    Nl80211StationInfo,
};

use crate::{Iface, IfaceType, NisporError, mac::parse_as_mac};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WifiInfo {
    pub mode: WifiMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bssid: Option<String>,
    /// Frequency in MHz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<u32>,
    /// WIFI generation, e.g. 6 for WIFI-6.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation: Option<u32>,
    /// WIFI signal in dBm
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<i8>,
    /// Receive bitrate in 100kb/s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_bitrate: Option<u32>,
    /// Receive channel width in MHz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_width: Option<u32>,
    /// Transmit bitrate in 100kb/s.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_bitrate: Option<u32>,
    /// Transmit channel width in MHz
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_width: Option<u32>,
}

pub(crate) async fn fill_wifi_info(
    iface_states: &mut HashMap<String, Iface>,
) -> Result<(), NisporError> {
    let mut wifi_ifaces: Vec<String> = Vec::new();
    let (connection, handle, _) = wl_nl80211::new_connection()?;
    tokio::spawn(connection);

    let mut interface_handle =
        handle.interface().get(Vec::new()).execute().await;

    while let Some(msg) = interface_handle.try_next().await? {
        let attrs = &msg.payload.attributes;
        let iface_name = if let Some(iface_name) =
            attrs.iter().find_map(|attr| {
                if let Nl80211Attr::IfName(n) = attr {
                    Some(n)
                } else {
                    None
                }
            }) {
            iface_name
        } else {
            continue;
        };
        let iface = if let Some(i) = iface_states.get_mut(iface_name) {
            i
        } else {
            continue;
        };
        let mut info = WifiInfo::default();
        for attr in attrs {
            match attr {
                Nl80211Attr::WiphyFreq(f) => info.frequency = Some(*f),
                Nl80211Attr::Ssid(s) if !s.is_empty() => {
                    info.ssid = Some(s.to_string())
                }
                Nl80211Attr::IfType(t) => info.mode = (*t).into(),
                _ => (),
            }
        }
        if info.mode == WifiMode::Ap
            && let Some(Nl80211Attr::Mac(mac)) =
                attrs.iter().find(|a| matches!(a, Nl80211Attr::Mac(_)))
        {
            info.bssid = parse_as_mac(mac.len(), mac.as_slice()).ok();
        }
        iface.wifi = Some(info);
        iface.iface_type = IfaceType::Wifi;
        wifi_ifaces.push(iface.name.clone());
    }

    for iface_name in wifi_ifaces {
        let iface = if let Some(i) = iface_states.get_mut(&iface_name) {
            i
        } else {
            continue;
        };
        let wifi = if let Some(w) = iface.wifi.as_mut() {
            w
        } else {
            continue;
        };
        if wifi.mode != WifiMode::Station {
            continue;
        }

        let mut station_handle =
            handle.station().dump(iface.index).execute().await;

        // 802.11g connection in kernel does not have SSID stored in reply of
        // handle.interface().get()
        // We need to use station MAC and scan results instead
        let mac_to_ssid = if wifi.ssid.as_deref().is_none_or(str::is_empty) {
            get_mac_ssid_map(&handle, iface.index).await?
        } else {
            HashMap::new()
        };

        let mut authorized = false;

        while let Some(msg) = station_handle.try_next().await? {
            if let Some(station_mac) =
                msg.payload.attributes.as_slice().iter().find_map(|attr| {
                    if let Nl80211Attr::Mac(m) = attr {
                        Some(m)
                    } else {
                        None
                    }
                })
            {
                let mac_str = parse_as_mac(ETH_ALEN, station_mac.as_slice())?;
                wifi.bssid = Some(mac_str.to_string());
                if let Some(ssid) = mac_to_ssid.get(&mac_str) {
                    wifi.ssid = Some(ssid.to_string());
                }
            }
            let sta_infos = if let Some(sta_infos) =
                msg.payload.attributes.as_slice().iter().find_map(|attr| {
                    if let Nl80211Attr::StationInfo(infos) = attr {
                        Some(infos)
                    } else {
                        None
                    }
                }) {
                sta_infos
            } else {
                continue;
            };
            for sta_info in sta_infos {
                match sta_info {
                    Nl80211StationInfo::Signal(s) => wifi.signal = Some(*s),
                    Nl80211StationInfo::TxBitrate(rates) => {
                        for rate in rates {
                            match rate {
                                Nl80211RateInfo::Bitrate32(d) => {
                                    wifi.tx_bitrate = Some(*d)
                                }
                                Nl80211RateInfo::MhzWidth(d) => {
                                    wifi.tx_width = Some(*d)
                                }
                                _ => (),
                            }
                        }
                    }
                    Nl80211StationInfo::StationFlags(v)
                        if v.set.contains(Nl80211StationFlags::Authorized) =>
                    {
                        authorized = true;
                    }
                    Nl80211StationInfo::RxBitrate(rates) => {
                        for rate in rates {
                            match rate {
                                Nl80211RateInfo::Bitrate32(d) => {
                                    wifi.rx_bitrate = Some(*d)
                                }
                                Nl80211RateInfo::MhzWidth(d) => {
                                    wifi.rx_width = Some(*d)
                                }
                                Nl80211RateInfo::Mcs(_) => {
                                    wifi.generation = Some(4)
                                }
                                Nl80211RateInfo::VhtMcs(_) => {
                                    wifi.generation = Some(5)
                                }
                                Nl80211RateInfo::HeMcs(_) => {
                                    wifi.generation = Some(6)
                                }
                                Nl80211RateInfo::EhtMcs(_) => {
                                    wifi.generation = Some(7)
                                }
                                _ => (),
                            }
                        }
                    }
                    _ => (),
                }
            }
            // TODO(Gris Ge): Once wl_nl80211 support NL80211_ATTR_AUTH_TYPE,
            // we should not clean SSID for WEP or Open system
            if !authorized {
                wifi.ssid = None;
            }
        }
    }

    Ok(())
}

const ETH_ALEN: usize = 6;

async fn get_mac_ssid_map(
    handle: &Nl80211Handle,
    iface_index: u32,
) -> Result<HashMap<String, String>, NisporError> {
    let mut ret = HashMap::new();
    let mut scan_handle = handle.scan().dump(iface_index).execute().await;
    while let Some(msg) = scan_handle.try_next().await? {
        let mut ssid: Option<String> = None;
        let mut mac_str: Option<String> = None;
        for attr in msg.payload.attributes.as_slice() {
            if ssid.is_some() && mac_str.is_some() {
                break;
            }
            if let Nl80211Attr::Bss(bss_infos) = attr {
                for bss_info in bss_infos {
                    if ssid.is_some() && mac_str.is_some() {
                        break;
                    }
                    if let Nl80211BssInfo::Bssid(mac) = bss_info {
                        mac_str = Some(parse_as_mac(ETH_ALEN, mac)?);
                    } else if let Nl80211BssInfo::InformationElements(ies) =
                        bss_info
                    {
                        ssid = ies.iter().find_map(|ie| {
                            if let Nl80211Element::Ssid(s) = ie {
                                Some(s.to_string())
                            } else {
                                None
                            }
                        });
                    }
                }
            }
        }
        if let (Some(ssid), Some(mac_str)) = (ssid, mac_str) {
            ret.insert(mac_str, ssid);
        }
    }

    Ok(ret)
}

#[derive(
    Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy, Default,
)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub enum WifiMode {
    #[default]
    Station,
    Ap,
    Other(u32),
}

impl From<Nl80211InterfaceType> for WifiMode {
    fn from(v: Nl80211InterfaceType) -> Self {
        match v {
            Nl80211InterfaceType::Station => Self::Station,
            Nl80211InterfaceType::Ap => Self::Ap,
            _ => Self::Other(v.into()),
        }
    }
}
