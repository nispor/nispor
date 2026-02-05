// SPDX-License-Identifier: Apache-2.0

use std::net::SocketAddr;

use nl_wireguard::{WireguardHandle, WireguardParsed, WireguardPeerParsed};
use serde::{Deserialize, Serialize};

use crate::{IfaceConf, NisporError, WireguardIpAddress};

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardConf {
    /// Base64 encoded private key, will be shown as `<hidden>` for Debug and
    /// excluded from Serialize
    #[serde(skip_serializing)]
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

impl std::fmt::Debug for WireguardConf {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        f.debug_struct("WireguardConf")
            .field(
                "private_key",
                &self.private_key.as_ref().map(|_| "<hidden>"),
            )
            .field("listen_port", &self.listen_port)
            .field("fwmark", &self.fwmark)
            .field("peers", &self.peers)
            .finish()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct WireguardPeerConf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<SocketAddr>,
    /// Base64 encoded public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_key: Option<String>,
    /// Base64 encoded preshared key, will be shown as `<hidden>` for Debug and
    /// excluded from Serialize
    #[serde(skip_serializing)]
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

impl std::fmt::Debug for WireguardPeerConf {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> Result<(), std::fmt::Error> {
        f.debug_struct("WireguardPeerConf")
            .field("endpoint", &self.endpoint)
            .field("public_key", &self.public_key)
            .field(
                "preshared_key",
                &self.preshared_key.as_ref().map(|_| "<hidden>"),
            )
            .field("persistent_keepalive", &self.persistent_keepalive)
            .field("allowed_ips", &self.allowed_ips)
            .field("protocol_version", &self.protocol_version)
            .finish()
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_hide_secrets_wg_conf() {
        let conf = WireguardConf {
            private_key: Some("top_secrets".into()),
            listen_port: Some(12123),
            ..Default::default()
        };

        let debug_output = format!("{conf:?}");

        assert!(debug_output.contains("WireguardConf"));
        assert!(debug_output.contains("listen_port"));
        assert!(debug_output.contains("12123"));
        assert!(debug_output.contains("private_key"));
        assert!(debug_output.contains("<hidden>"));
        assert!(!debug_output.contains("top_secrets"));
    }

    #[test]
    fn test_hide_secrets_wg_peer_conf() {
        let conf = WireguardPeerConf {
            preshared_key: Some("top_secrets".into()),
            public_key: Some("ok_to_share".into()),
            ..Default::default()
        };

        let debug_output = format!("{conf:?}");

        assert!(debug_output.contains("WireguardPeerConf"));
        assert!(debug_output.contains("public_key"));
        assert!(debug_output.contains("ok_to_share"));
        assert!(debug_output.contains("preshared_key"));
        assert!(debug_output.contains("<hidden>"));
        assert!(!debug_output.contains("top_secrets"));
    }
}
