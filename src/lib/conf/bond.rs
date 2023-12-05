// SPDX-License-Identifier: Apache-2.0

use netlink_packet_route::link::{
    InfoBond, InfoData, InfoKind, LinkAttribute, LinkInfo,
};
use rtnetlink::Handle;
use serde::{Deserialize, Serialize};

use crate::{BondMode, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondConf {
    pub mode: Option<BondMode>,
}

impl BondConf {
    pub(crate) async fn create(
        &self,
        handle: &Handle,
        name: &str,
    ) -> Result<(), NisporError> {
        // Unlink bridge, rust-rtnetlink does not support bond creation out of
        // box.
        let mut req = handle.link().add();
        let mutator = req.message_mut();

        let mode = self.mode.unwrap_or_default();
        let info = LinkAttribute::LinkInfo(vec![
            LinkInfo::Kind(InfoKind::Bond),
            LinkInfo::Data(InfoData::Bond(vec![InfoBond::Mode(mode.into())])),
        ]);
        mutator.attributes.push(info);
        mutator
            .attributes
            .push(LinkAttribute::IfName(name.to_string()));

        match req.execute().await {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new bond '{}': {}",
                &name, e
            ))),
        }
    }
}
