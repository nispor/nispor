// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkMessageBuilder, LinkVeth};

use crate::{ErrorKind, IfaceConf, NisporError, VethInfo};

pub type VethConf = VethInfo;

impl VethConf {
    pub(crate) fn create(
        iface: &IfaceConf,
    ) -> Result<LinkMessageBuilder<LinkVeth>, NisporError> {
        if let Some(veth_conf) = &iface.veth {
            Ok(LinkVeth::new(iface.name.as_str(), veth_conf.peer.as_str()))
        } else {
            Err(NisporError::new(
                ErrorKind::InvalidArgument,
                "No veth peer defined to creating veth".to_string(),
            ))
        }
    }
}
