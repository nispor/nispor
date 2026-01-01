// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{packet_route::link::LinkMessage, LinkBondPort};
use serde::{Deserialize, Serialize};

use crate::Iface;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(deny_unknown_fields)]
pub struct BondPortConf {
    pub queue_id: Option<u16>,
    pub prio: Option<i32>,
}

impl BondPortConf {
    pub(crate) fn gen_link_msg(&self, cur_iface: &Iface) -> LinkMessage {
        let mut builder = LinkBondPort::new(cur_iface.index);

        if let Some(v) = self.queue_id {
            builder = builder.queue_id(v);
        }

        if let Some(v) = self.prio {
            builder = builder.prio(v);
        }

        builder.build()
    }
}
