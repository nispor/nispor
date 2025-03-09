// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{Handle, LinkBridge};
use serde::{Deserialize, Serialize};

use crate::NisporError;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BridgeConf {}

impl BridgeConf {
    pub(crate) async fn create(
        handle: &Handle,
        name: &str,
    ) -> Result<(), NisporError> {
        match handle
            .link()
            .add(LinkBridge::new(name).up().build())
            .execute()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(NisporError::bug(format!(
                "Failed to create new bridge '{}': {}",
                &name, e
            ))),
        }
    }
}
