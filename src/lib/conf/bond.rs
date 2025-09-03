// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkBond, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::{BondMode, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondConf {
    pub mode: Option<BondMode>,
}

impl BondConf {
    pub(crate) fn create(
        iface: &IfaceConf,
    ) -> Result<LinkMessageBuilder<LinkBond>, NisporError> {
        let bond_mode =
            iface.bond.as_ref().and_then(|b| b.mode).unwrap_or_default();
        Ok(LinkBond::new(iface.name.as_str()).mode(bond_mode.into()))
    }
}
