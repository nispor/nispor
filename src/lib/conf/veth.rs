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
            if let Some(cur_veth_peer) = cur_iface
                .as_ref()
                .and_then(|c| c.veth.as_ref())
                .map(|v| v.peer.as_str())
            {
                if !veth_conf.peer.is_empty()
                    // When veth peer is creating, the veth peer is a index
                    // number don't have name yet. In that case we don't fail.
                    && cur_veth_peer.parse::<i32>().is_err()
                    && veth_conf.peer.as_str() != cur_veth_peer
                {
                    Err(NisporError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Cannot change veth interface {} peer from {} to \
                             {} without removing a veth first",
                            iface.name, cur_veth_peer, veth_conf.peer
                        ),
                    ))
                } else {
                    Ok(LinkMessageBuilder::<LinkVeth>::new_with_info_kind(
                        InfoKind::Veth,
                    )
                    .name(iface.name.to_string()))
                }
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
