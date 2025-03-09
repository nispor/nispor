// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{Handle, LinkUnspec, LinkVeth};

use crate::{NisporError, VethInfo};

pub type VethConf = VethInfo;

impl VethConf {
    pub(crate) async fn create(
        &self,
        handle: &Handle,
        name: &str,
    ) -> Result<(), NisporError> {
        if let Err(e) = handle
            .link()
            .add(LinkVeth::new(name, self.peer.as_str()).up().build())
            .execute()
            .await
        {
            return Err(NisporError::bug(format!(
                "Failed to create new veth pair '{}' '{}': {}",
                &name, &self.peer, e
            )));
        }

        if let Err(e) = handle
            .link()
            .set(LinkUnspec::new_with_name(self.peer.as_str()).up().build())
            .execute()
            .await
        {
            return Err(NisporError::bug(format!(
                "Failed to bring veth pair up '{}' '{}': {}",
                &name, &self.peer, e
            )));
        }

        Ok(())
    }
}
