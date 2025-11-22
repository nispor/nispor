// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkDummy, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::{IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct DummyConf;

impl DummyConf {
    pub(crate) fn create(
        iface: &IfaceConf,
    ) -> Result<LinkMessageBuilder<LinkDummy>, NisporError> {
        Ok(LinkDummy::new(iface.name.as_str()))
    }
}
