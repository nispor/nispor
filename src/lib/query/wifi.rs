// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use wl_nl80211::{Nl80211Attr, Nl80211RateInfo, Nl80211StationInfo};

use crate::{Iface, IfaceType, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct WifiInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ssid: Option<String>,
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

    let mut interface_handle = handle.interface().get().execute().await;

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
                Nl80211Attr::Ssid(s) => info.ssid = Some(s.to_string()),
                _ => (),
            }
        }
        // TODO: Use station info to set BSSID(MAC of Station) and signal level.
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
        let mut station_handle =
            handle.station().dump(iface.index).execute().await;
        let wifi = if let Some(w) = iface.wifi.as_mut() {
            w
        } else {
            continue;
        };
        while let Some(msg) = station_handle.try_next().await? {
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
        }
    }

    Ok(())
}
