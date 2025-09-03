// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkMessageBuilder, LinkUnspec};

use super::super::{mac::mac_str_to_raw, query::resolve_iface_index};
use crate::{ErrorKind, Iface, IfaceConf, IfaceState, NisporError};

pub(crate) async fn apply_base_link_changes(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: &Iface,
) -> Result<(), NisporError> {
    let mut msg_builder = LinkUnspec::new_with_name(des_iface.name.as_str());
    let mut cur_iface_state = cur_iface.state.clone();
    if let Some(mac) = des_iface.mac_address.as_deref() {
        if cur_iface.mac_address.as_str() != mac {
            // Need to bring interface down to change the MAC
            if cur_iface_state != IfaceState::Down {
                link_down(handle, cur_iface.index).await?;
                cur_iface_state = IfaceState::Down;
            }
            msg_builder = apply_mac_change(msg_builder, des_iface)?;
        }
    }
    if let Some(des_ctrl) = des_iface.controller.as_ref() {
        if Some(des_ctrl) != cur_iface.controller.as_ref() {
            // Need down interface for changing controller
            if cur_iface_state != IfaceState::Down {
                link_down(handle, cur_iface.index).await?;
                cur_iface_state = IfaceState::Down;
            }
            apply_controller_change(handle, des_iface.name.as_str(), des_ctrl)
                .await?;
        }
    }
    if cur_iface_state != des_iface.state {
        msg_builder = apply_state_change(msg_builder, des_iface)?;
    }
    if let Some(mtu) = des_iface.mtu {
        if cur_iface.mtu != mtu as i64 {
            msg_builder = msg_builder.mtu(mtu);
        }
    }

    handle
        .link()
        .set(msg_builder.build())
        .execute()
        .await
        .map_err(|e| {
            NisporError::new(
                ErrorKind::NisporBug,
                format!("Failed to change interface {des_iface:?}: {e}"),
            )
        })
}

fn apply_state_change<T>(
    msg_builder: LinkMessageBuilder<T>,
    iface: &IfaceConf,
) -> Result<LinkMessageBuilder<T>, NisporError> {
    match iface.state {
        IfaceState::Up => Ok(msg_builder.up()),
        IfaceState::Down => Ok(msg_builder.down()),
        IfaceState::Absent => Err(NisporError::new(
            ErrorKind::NisporBug,
            format!(
                "apply_state_change() got IfaceState::Absent which should \
                 never reach here: {iface:?}",
            ),
        )),
        _ => Err(NisporError::new(
            ErrorKind::InvalidArgument,
            format!(
                "Invalid interface state {} which should never reach here",
                iface.state
            ),
        )),
    }
}

fn apply_mac_change<T>(
    msg_builder: LinkMessageBuilder<T>,
    iface: &IfaceConf,
) -> Result<LinkMessageBuilder<T>, NisporError> {
    if let Some(mac_address) = iface.mac_address.as_deref() {
        Ok(msg_builder.address(mac_str_to_raw(mac_address)?))
    } else {
        Ok(msg_builder)
    }
}

/// `ctrl_name.is_empty()` means detach
async fn apply_controller_change(
    handle: &rtnetlink::Handle,
    iface_name: &str,
    ctrl_name: &str,
) -> Result<(), NisporError> {
    let msg_builder = if ctrl_name.is_empty() {
        LinkUnspec::new_with_name(iface_name).nocontroller()
    } else {
        let ctrl_index = resolve_iface_index(handle, ctrl_name).await?;
        LinkUnspec::new_with_name(iface_name).controller(ctrl_index)
    };

    handle
        .link()
        .set(msg_builder.build())
        .execute()
        .await
        .map_err(|e| {
            NisporError::new(
                ErrorKind::NisporBug,
                format!(
                    "Failed to change interface {iface_name} controller \
                     {ctrl_name}: {e}"
                ),
            )
        })
}

async fn link_down(
    handle: &rtnetlink::Handle,
    index: u32,
) -> Result<(), NisporError> {
    log::debug!("Bring interface {index} down");
    handle
        .link()
        .set(LinkUnspec::new_with_index(index).down().build())
        .execute()
        .await
        .map_err(|e| {
            NisporError::new(
                ErrorKind::NisporBug,
                format!("Failed to link down iface index {index}: {e}"),
            )
        })
}
