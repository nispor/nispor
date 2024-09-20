// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use wl_nl80211::Nl80211Attr;

use crate::{Iface, IfaceType, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub struct WifiInfo {
    pub ssid: Option<String>,
    /// Frequency in MHz
    pub frequency: Option<u32>,
    /// WIFI generation, e.g. 6 for WIFI-6.
    pub generation: Option<u32>,
}

pub(crate) async fn fill_wifi_info(
    iface_states: &mut HashMap<String, Iface>,
) -> Result<(), NisporError> {
    let (connection, handle, _) = wl_nl80211::new_connection()?;
    tokio::spawn(connection);

    let mut interface_handle = handle.interface().get().execute().await;

    while let Some(msg) = interface_handle.try_next().await? {
        let attrs = &msg.payload.nlas;
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
                Nl80211Attr::Generation(g) => info.generation = Some(*g),
                Nl80211Attr::WiPhyFreq(f) => info.frequency = Some(*f),
                Nl80211Attr::Ssid(s) => info.ssid = Some(s.to_string()),
                _ => (),
            }
        }
        // TODO: Use station info to set BSSID(MAC of Station) and signal level.
        iface.wifi = Some(info);
        iface.iface_type = IfaceType::Wifi;
    }
    Ok(())
}
