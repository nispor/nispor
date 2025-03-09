// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{Handle, LinkBond};
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
        let mode: u8 = self.mode.unwrap_or_default().into();
        let req = handle
            .link()
            .add(LinkBond::new(name).mode(mode.into()).up().build());

        match req.execute().await {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new bond '{}': {}",
                &name, e
            ))),
        }
    }
}
