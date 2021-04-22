use std::collections::HashMap;

use futures::stream::TryStreamExt;
use netlink_ethtool::{self, EthoolAttr, EthtoolHeader, PauseAttr};
use netlink_generic;
use serde_derive::{Deserialize, Serialize};

use crate::NisporError;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
pub struct EthtoolPauseInfo {
    rx: bool,
    tx: bool,
    auto_negotiate: bool,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
pub struct EthtoolInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pause: Option<EthtoolPauseInfo>,
}

pub(crate) async fn get_ethtool_infos(
) -> Result<HashMap<String, EthtoolInfo>, NisporError> {
    let mut infos: HashMap<String, EthtoolInfo> = HashMap::new();

    let family_id = get_family_id().await?;

    let (connection, mut handle, _) =
        netlink_ethtool::new_connection(family_id).unwrap();
    tokio::spawn(connection);

    for ethtool_msg in handle.pause().get(None).execute().try_next().await? {
        let EthoolAttr::Pause(nlas) = ethtool_msg.nlas;
        let mut iface_name = None;
        let mut pause_info = EthtoolPauseInfo::default();

        for nla in &nlas {
            if let PauseAttr::Header(hdrs) = nla {
                iface_name = get_iface_name_from_header(&hdrs);
            } else if let PauseAttr::AutoNeg(v) = nla {
                pause_info.auto_negotiate = *v
            } else if let PauseAttr::Rx(v) = nla {
                pause_info.rx = *v
            } else if let PauseAttr::Tx(v) = nla {
                pause_info.tx = *v
            }
        }
        if let Some(i) = iface_name {
            infos.insert(
                i.to_string(),
                EthtoolInfo {
                    pause: Some(pause_info),
                },
            );
        }
    }

    Ok(infos)
}

async fn get_family_id() -> Result<u16, NisporError> {
    let (connection, mut handle, _) =
        netlink_generic::new_connection().unwrap();
    tokio::spawn(connection);

    Ok(handle.resolve_family_name("ethtool").await?)
}

fn get_iface_name_from_header(hdrs: &[EthtoolHeader]) -> Option<&str> {
    for hdr in hdrs {
        if let EthtoolHeader::DevName(iface_name) = hdr {
            return Some(iface_name.as_str());
        }
    }
    None
}
