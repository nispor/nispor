// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;

use nl_wireguard::{WireguardHandle, WireguardParsed, WireguardPeerParsed};
use serde::{Deserialize, Serialize};

use crate::{IfaceConf, NisporError, WireguardIpAddress};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardConf {
    /// Base64 encoded private key
    pub private_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fwmark: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peers: Option<Vec<WireguardPeerConf>>,
}

impl From<&WireguardConf> for WireguardParsed {
    fn from(conf: &WireguardConf) -> WireguardParsed {
        let mut wg_conf = WireguardParsed::default();
        wg_conf.private_key = conf.private_key.clone();
        wg_conf.listen_port = conf.listen_port;
        wg_conf.fwmark = conf.fwmark;
        wg_conf.peers = conf
            .peers
            .as_ref()
            .map(|peers| peers.iter().map(WireguardPeerParsed::from).collect());
        wg_conf
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardPeerConf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<SocketAddr>,
    /// Base64 encoded public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Base64 encoded preshared key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preshared_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub persistent_keepalive: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_ips: Option<Vec<WireguardIpAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol_version: Option<u32>,
}

impl From<&WireguardPeerConf> for WireguardPeerParsed {
    fn from(peer_conf: &WireguardPeerConf) -> Self {
        let mut wg_peer_conf = Self::default();
        wg_peer_conf.endpoint = peer_conf.endpoint;
        wg_peer_conf.public_key = peer_conf.public_key.clone();
        wg_peer_conf.preshared_key = peer_conf.preshared_key.clone();
        wg_peer_conf.persistent_keepalive = peer_conf.persistent_keepalive;
        wg_peer_conf.protocol_version = peer_conf.protocol_version;
        wg_peer_conf.allowed_ips = peer_conf.allowed_ips.as_ref().map(|ips| {
            ips.iter()
                .map(|ip| nl_wireguard::WireguardIpAddress::from(*ip))
                .collect()
        });
        wg_peer_conf
    }
}

pub(crate) async fn apply_wg_conf(
    handle: &mut WireguardHandle,
    iface: &IfaceConf,
) -> Result<(), NisporError> {
    if let Some(wg_conf) = iface.wireguard.as_ref() {
        let mut conf = WireguardParsed::from(wg_conf);
        conf.iface_name = Some(iface.name.to_string());
        handle.set(conf).await?;
    }

    Ok(())
}
