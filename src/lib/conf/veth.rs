// SPDX-License-Identifier: Apache-2.0

use rtnetlink::Handle;

use crate::{NisporError, VethInfo};

pub type VethConf = VethInfo;

impl VethConf {
    pub(crate) async fn create(
        &self,
        handle: &Handle,
        name: &str,
    ) -> Result<(), NisporError> {
        match handle
            .link()
            .add()
            .veth(name.to_string(), self.peer.clone())
            .execute()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new veth pair '{}' '{}': {}",
                &name, &self.peer, e
            ))),
        }
    }
}
