// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{Handle, LinkMessageBuilder, LinkVlan};
use serde::{Deserialize, Serialize};

use super::super::query::resolve_iface_index;
use crate::{ErrorKind, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VlanConf {
    pub vlan_id: u16,
    pub base_iface: String,
}

impl VlanConf {
    pub(crate) async fn create(
        handle: &Handle,
        iface: &IfaceConf,
    ) -> Result<LinkMessageBuilder<LinkVlan>, NisporError> {
        if let Some(vlan_conf) = iface.vlan.as_ref() {
            let parent_index =
                resolve_iface_index(handle, &vlan_conf.base_iface).await?;
            Ok(LinkVlan::new(
                iface.name.as_str(),
                parent_index,
                vlan_conf.vlan_id,
            ))
        } else {
            Err(NisporError::new(
                ErrorKind::NisporBug,
                format!("No vlan section defined for creating VLAN {iface:?}"),
            ))
        }
    }
}
