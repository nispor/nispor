// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{super::mac::mac_str_to_raw, inter_ifaces::change_ifaces};
use crate::{
    BondConf, BridgeConf, Iface, IfaceState, IfaceType, IpConf, NisporError,
    VethConf, VlanConf,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct IfaceConf {
    pub name: String,
    #[serde(default = "default_iface_state_in_conf")]
    pub state: IfaceState,
    #[serde(rename = "type")]
    pub iface_type: Option<IfaceType>,
    pub controller: Option<String>,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
    pub mac_address: Option<String>,
    pub veth: Option<VethConf>,
    pub bridge: Option<BridgeConf>,
    pub vlan: Option<VlanConf>,
    pub bond: Option<BondConf>,
}

impl IfaceConf {
    pub async fn apply(&self, cur_iface: &Iface) -> Result<(), NisporError> {
        log::warn!(
            "WARN: IfaceConf::apply() is deprecated, \
            please use NetConf::apply() instead"
        );
        let ifaces = vec![self];
        let mut cur_ifaces = HashMap::new();
        cur_ifaces.insert(self.name.to_string(), cur_iface.clone());
        change_ifaces(&ifaces, &cur_ifaces).await
    }
}

fn default_iface_state_in_conf() -> IfaceState {
    IfaceState::Up
}

pub(crate) async fn change_iface_state(
    handle: &rtnetlink::Handle,
    index: u32,
    up: bool,
) -> Result<(), NisporError> {
    if up {
        handle.link().set(index).up().execute().await?;
    } else {
        handle.link().set(index).down().execute().await?;
    }
    Ok(())
}

pub(crate) async fn change_iface_mac(
    handle: &rtnetlink::Handle,
    index: u32,
    mac_address: &str,
) -> Result<(), NisporError> {
    change_iface_state(handle, index, false).await?;
    handle
        .link()
        .set(index)
        .address(mac_str_to_raw(mac_address)?)
        .execute()
        .await?;
    Ok(())
}
