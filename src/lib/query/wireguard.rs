// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType, NisporError};

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardInfo {
    /// Base64 encoded public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fwmark: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<WireguardPeerInfo>>,
}

impl From<nl_wireguard::WireguardParsed> for WireguardInfo {
    fn from(nl_wg: nl_wireguard::WireguardParsed) -> Self {
        Self {
            public_key: nl_wg.public_key,
            listen_port: nl_wg.listen_port,
            fwmark: nl_wg.fwmark,
            peers: nl_wg.peers.map(|nl_peers| {
                nl_peers.into_iter().map(WireguardPeerInfo::from).collect()
            }),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardPeerInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Base64 encoded public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Whether has pershared key configure or not
    pub has_preshared_key: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_keepalive: Option<u16>,
    /// Last handshake in a format of `32 seconds ago`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_handshake: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rx_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<WireguardIpAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<u32>,
}

impl From<nl_wireguard::WireguardPeerParsed> for WireguardPeerInfo {
    fn from(nl_peer: nl_wireguard::WireguardPeerParsed) -> Self {
        Self {
            endpoint: nl_peer.endpoint.map(|e| e.to_string()),
            public_key: nl_peer.public_key,
            has_preshared_key: nl_peer.preshared_key.is_some(),
            persistent_keepalive: nl_peer.persistent_keepalive,
            last_handshake: nl_peer
                .last_handshake
                .and_then(last_handshake_to_human_str),
            rx_bytes: nl_peer.rx_bytes,
            tx_bytes: nl_peer.tx_bytes,
            allowed_ips: nl_peer.allowed_ips.map(|nl_ips| {
                nl_ips.into_iter().map(WireguardIpAddress::from).collect()
            }),
            protocol_version: nl_peer.protocol_version,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Copy)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct WireguardIpAddress {
    pub address: IpAddr,
    pub prefix_length: u8,
}

impl From<nl_wireguard::WireguardIpAddress> for WireguardIpAddress {
    fn from(nl_ip: nl_wireguard::WireguardIpAddress) -> Self {
        Self {
            address: nl_ip.ip_addr,
            prefix_length: nl_ip.prefix_length,
        }
    }
}

impl From<WireguardIpAddress> for nl_wireguard::WireguardIpAddress {
    fn from(ip: WireguardIpAddress) -> Self {
        Self {
            ip_addr: ip.address,
            prefix_length: ip.prefix_length,
        }
    }
}

pub(crate) async fn fill_wireguard(
    ifaces: &mut HashMap<String, Iface>,
) -> Result<(), NisporError> {
    if !ifaces
        .values()
        .any(|i| i.iface_type == IfaceType::Wireguard)
    {
        return Ok(());
    }

    let (connection, mut handle, _) = nl_wireguard::new_connection()?;
    tokio::spawn(connection);

    for iface in ifaces
        .values_mut()
        .filter(|i| i.iface_type == IfaceType::Wireguard)
    {
        match handle.get_by_name(iface.name.as_str()).await {
            Ok(nl_wg) => iface.wireguard = Some(nl_wg.into()),
            Err(e) => {
                // Mostly permission error because querying wireguard info need
                // root access.
                log::debug!("Failed to fill wireguard information: {e}");
            }
        }
    }

    Ok(())
}

fn last_handshake_to_human_str(last_handshake: Duration) -> Option<String> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?;

    if last_handshake > now {
        None
    } else {
        Some(format!("{} seconds ago", (now - last_handshake).as_secs()))
    }
}
