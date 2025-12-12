// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkBond, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::{BondMode, IfaceConf};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondConf {
    pub mode: Option<BondMode>,
}

impl BondConf {
    pub(crate) fn gen_link_msg_builder(
        iface: &IfaceConf,
    ) -> LinkMessageBuilder<LinkBond> {
        let mut builder = LinkBond::new(iface.name.as_str());
        if let Some(bond_mode) = iface.bond.as_ref().and_then(|b| b.mode) {
            builder = builder.mode(bond_mode.into());
        }
        builder
    }
}
