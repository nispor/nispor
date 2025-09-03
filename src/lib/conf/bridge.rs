// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkBridge, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::IfaceConf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BridgeConf {}

impl BridgeConf {
    pub(crate) fn create(iface: &IfaceConf) -> LinkMessageBuilder<LinkBridge> {
        LinkBridge::new(iface.name.as_str())
    }
}
