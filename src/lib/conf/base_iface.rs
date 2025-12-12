// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_route::link::LinkMessage, LinkMessageBuilder, LinkUnspec,
};

use super::super::{mac::mac_str_to_raw, query::resolve_iface_index};
use crate::{ErrorKind, Iface, IfaceConf, IfaceState, NisporError};

pub(crate) async fn apply_base_link_changes<T>(
    handle: &rtnetlink::Handle,
    mut builder: LinkMessageBuilder<T>,
    des_iface: &IfaceConf,
    cur_iface: Option<&Iface>,
) -> Result<Vec<LinkMessage>, NisporError> {
    let mut ret: Vec<LinkMessage> = Vec::new();

    // We cannot bring interface up when changing controller, hence
    // we need to bring interface up after controller changed.
    let mut is_changing_ctrller = false;
    let mut post_apply_msg: Option<LinkMessage> = None;

    // Interface is created with DOWN state, so even current interface not
    // exist yet, its current state should be DOWN.
    let mut cur_iface_state = cur_iface
        .as_ref()
        .map(|c| c.state.clone())
        .unwrap_or(IfaceState::Down);

    if let Some(mtu) = des_iface.mtu {
        builder = builder.mtu(mtu);
    }

    if let Some(des_mac) = des_iface.mac_address.as_ref() {
        if !des_mac.is_empty() {
            if let Some(cur_iface) = cur_iface.as_ref() {
                if des_mac.to_uppercase()
                    != cur_iface.mac_address.as_str().to_uppercase()
                    && cur_iface_state != IfaceState::Down
                {
                    // Need to bring interface down to change the MAC
                    ret.push(
                        LinkUnspec::new_with_index(cur_iface.index)
                            .down()
                            .build(),
                    );
                    cur_iface_state = IfaceState::Down;
                }
            }
            builder = builder.address(mac_str_to_raw(des_mac)?);
        }
    }

    if let Some(des_ctrl) = des_iface.controller.as_ref() {
        if let Some(cur_ctrl) =
            cur_iface.as_ref().and_then(|c| c.controller.as_ref())
        {
            if des_ctrl != cur_ctrl {
                // Need to bring down interface for changing controller
                if cur_iface_state != IfaceState::Down {
                    ret.push(
                        LinkUnspec::new_with_name(des_iface.name.as_str())
                            .down()
                            .build(),
                    );
                    cur_iface_state = IfaceState::Down;
                }
            }
        }

        is_changing_ctrller = true;
        if des_ctrl.is_empty() {
            builder = builder.nocontroller();
        } else {
            let ctrl_index = resolve_iface_index(handle, des_ctrl).await?;
            builder = builder.controller(ctrl_index);
        }
    }

    // When changing controller, we cannot make the interface as up yet.
    if cur_iface_state != des_iface.state {
        match &des_iface.state {
            IfaceState::Up => {
                if is_changing_ctrller {
                    builder = builder.down();
                    post_apply_msg = Some(
                        LinkUnspec::new_with_name(des_iface.name.as_str())
                            .up()
                            .build(),
                    );
                } else {
                    builder = builder.up();
                }
            }
            IfaceState::Down => {
                builder = builder.down();
            }
            state => {
                return Err(NisporError::new(
                    ErrorKind::Bug,
                    format!(
                        "apply_base_link_changes(): Invalid interface state \
                         {state}"
                    ),
                ));
            }
        }
    }

    ret.push(builder.build());
    if let Some(msg) = post_apply_msg {
        ret.push(msg)
    }

    Ok(ret)
}
