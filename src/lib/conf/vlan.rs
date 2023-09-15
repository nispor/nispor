// SPDX-License-Identifier: Apache-2.0

use rtnetlink::Handle;
use serde::{Deserialize, Serialize};

use crate::NisporError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VlanConf {
    pub vlan_id: u16,
    pub base_iface: String,
}

impl VlanConf {
    pub(crate) async fn create(
        handle: &Handle,
        name: &str,
        vlan_id: u16,
        base_iface_index: u32,
    ) -> Result<(), NisporError> {
        match handle
            .link()
            .add()
            .vlan(name.to_string(), base_iface_index, vlan_id)
            .execute()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new vlan '{}': {}",
                &name, e
            ))),
        }
    }
}
