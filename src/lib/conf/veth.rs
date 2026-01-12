// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkMessageBuilder, LinkVeth, packet_route::link::InfoKind};

use crate::{ErrorKind, Iface, IfaceConf, NisporError, VethInfo};

pub type VethConf = VethInfo;

impl VethConf {
    pub(crate) fn gen_link_msg_builder(
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<LinkMessageBuilder<LinkVeth>, NisporError> {
        if let Some(veth_conf) = &iface.veth {
            if cur_iface.as_ref().and_then(|c| c.veth.as_ref()).is_some() {
                Err(NisporError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Please remove veth section since veth interface {} \
                         already exists",
                        iface.name
                    ),
                ))
            } else {
                Ok(LinkVeth::new(iface.name.as_str(), veth_conf.peer.as_str()))
            }
        } else {
            Ok(LinkMessageBuilder::<LinkVeth>::new_with_info_kind(
                InfoKind::Veth,
            )
            .name(iface.name.to_string()))
        }
    }
}
