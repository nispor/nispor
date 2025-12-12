// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkDummy, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::IfaceConf;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct DummyConf {}

impl DummyConf {
    pub(crate) fn gen_link_msg_builder(
        iface: &IfaceConf,
    ) -> LinkMessageBuilder<LinkDummy> {
        LinkDummy::new(iface.name.as_str())
    }
}
