// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_route::link::LinkMessage, LinkMessageBuilder, LinkUnspec,
};

use super::super::{mac::mac_str_to_raw, query::resolve_iface_index};
use crate::{Iface, IfaceConf, IfaceFlag, IfaceState, NisporError};

pub(crate) async fn apply_base_link_changes<T>(
    handle: &rtnetlink::Handle,
    mut builder: LinkMessageBuilder<T>,
    des_iface: &IfaceConf,
    cur_iface: Option<&Iface>,
) -> Result<Vec<LinkMessage>, NisporError> {
    let mut ret: Vec<LinkMessage> = Vec::new();

    if let Some(cur_iface) = cur_iface {
        if des_iface.need_state_down_before_apply(cur_iface)
            && cur_iface.flags.contains(&IfaceFlag::Up)
        {
            ret.push(
                LinkUnspec::new_with_index(cur_iface.index).down().build(),
            );
        }
    }

    if let Some(mtu) = des_iface.mtu {
        builder = builder.mtu(mtu);
    }

    if let Some(des_mac) = des_iface.mac_address.as_ref() {
        if !des_mac.is_empty() {
            let should_set = cur_iface
                .as_ref()
                .map(|c| c.mac_address.to_uppercase() != des_mac.to_uppercase())
                .unwrap_or(true);

            if should_set {
                builder = builder.address(mac_str_to_raw(des_mac)?);
            }
        }
    }

    if let Some(des_ctrl) = des_iface.controller.as_ref() {
        if let Some(cur_ctrl) =
            cur_iface.as_ref().and_then(|c| c.controller.as_ref())
        {
            if des_ctrl != cur_ctrl {
                if des_ctrl.is_empty() {
                    builder = builder.nocontroller();
                } else {
                    let ctrl_index =
                        resolve_iface_index(handle, des_ctrl).await?;
                    builder = builder.controller(ctrl_index);
                }
            }
        } else if des_ctrl.is_empty() {
            builder = builder.nocontroller();
        } else {
            let ctrl_index = resolve_iface_index(handle, des_ctrl).await?;
            builder = builder.controller(ctrl_index);
        }
    }

    ret.push(builder.build());

    // Many changes cannot done along with link up/down flag set(e.g.
    // change controller, change bond mode), so we change interface state
    // in a separate link message
    match &des_iface.state {
        IfaceState::Up => {
            ret.push(
                LinkUnspec::new_with_name(des_iface.name.as_str())
                    .up()
                    .build(),
            );
        }
        IfaceState::Down => {
            ret.push(
                LinkUnspec::new_with_name(des_iface.name.as_str())
                    .down()
                    .build(),
            );
        }
        _ => (),
    }

    Ok(ret)
}
