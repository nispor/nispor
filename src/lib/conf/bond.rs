// SPDX-License-Identifier: Apache-2.0

use netlink_packet_route::link::nlas::{
    Info, InfoBond, InfoData, InfoKind, Nla,
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
        let info = Nla::Info(vec![
            Info::Kind(InfoKind::Bond),
            Info::Data(InfoData::Bond(vec![InfoBond::Mode(mode.into())])),
        ]);
        mutator.nlas.push(info);
        mutator.nlas.push(Nla::IfName(name.to_string()));

        match req.execute().await {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new bond '{}': {}",
                &name, e
            ))),
        }
    }
}
