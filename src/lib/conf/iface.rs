// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_core::{NLM_F_ACK, NLM_F_REQUEST},
    packet_route::link::LinkMessage,
    LinkUnspec,
};
use serde::{Deserialize, Serialize};

use super::{
    alt_name::change_iface_alt_name, base_iface::apply_base_link_changes,
    ip::change_ip_layer,
};
use crate::{
    AltNameConf, BondConf, BridgeConf, DummyConf, ErrorKind, Iface, IfaceState,
    IfaceType, IpConf, NisporError, VethConf, VlanConf,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct IfaceConf {
    pub name: String,
    #[serde(default = "default_iface_state_in_conf")]
    pub state: IfaceState,
    #[serde(rename = "type")]
    pub iface_type: Option<IfaceType>,
    #[serde(default)]
    pub alt_names: Vec<AltNameConf>,
    /// Setting to Some(String::new()) will detach from controller
    pub controller: Option<String>,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
    pub mac_address: Option<String>,
    pub mtu: Option<u32>,
    pub veth: Option<VethConf>,
    pub bridge: Option<BridgeConf>,
    pub vlan: Option<VlanConf>,
    #[serde(alias = "link-aggregation")]
    pub bond: Option<BondConf>,
}

impl IfaceConf {
    /// Need interfaces to be down state for these changes:
    ///  * Change MAC address
    ///  * Change controller
    ///  * Change bond mode
    pub(crate) fn need_state_down_before_apply(&self, current: &Iface) -> bool {
        if let Some(des_mac) = self.mac_address.as_ref() {
            if des_mac.to_uppercase() != current.mac_address.to_uppercase() {
                return true;
            }
        }

        if self.controller.is_some() && self.controller != current.controller {
            return true;
        }

        if let Some(des_bond_mode) = self.bond.as_ref().and_then(|b| b.mode) {
            if let Some(cur_bond_mode) = current.bond.as_ref().map(|b| b.mode) {
                if des_bond_mode != cur_bond_mode {
                    return true;
                }
            }
        }

        false
    }
}

fn default_iface_state_in_conf() -> IfaceState {
    IfaceState::Up
}

pub(crate) async fn apply_iface_conf(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: Option<&Iface>,
) -> Result<(), NisporError> {
    if des_iface.state == IfaceState::Absent {
        if let Some(cur_iface) = cur_iface {
            delete_iface(handle, &des_iface.name, cur_iface.index).await?;
        } else {
            log::debug!(
                "Absent interface {} no found, no action",
                des_iface.name
            );
        }
    } else {
        let mut msgs = gen_link_msg(handle, des_iface, cur_iface).await?;
        if cur_iface.is_some() {
            for msg in msgs {
                send_change_netlink(handle, msg, des_iface.name.as_str())
                    .await?;
            }
        } else {
            if !msgs.is_empty() {
                let msg = msgs.remove(0);
                log::trace!(
                    "Creating interface {}/{} by netlink message {msg:?}",
                    des_iface.name,
                    des_iface.iface_type.clone().unwrap_or_default()
                );
                handle.link().add(msg).execute().await.map_err(|e| {
                    NisporError::new(
                        ErrorKind::NisporBug,
                        format!(
                            "Failed to create interface {des_iface:?}: {e}"
                        ),
                    )
                })?;
            }
            for msg in msgs {
                send_change_netlink(handle, msg, des_iface.name.as_str())
                    .await?;
            }
        }

        change_iface_alt_name(handle, des_iface, cur_iface).await?;
        change_ip_layer(handle, des_iface).await?;
    }

    Ok(())
}

async fn gen_link_msg(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: Option<&Iface>,
) -> Result<Vec<LinkMessage>, NisporError> {
    Ok(match des_iface.iface_type.as_ref() {
        Some(IfaceType::Bridge) => {
            apply_base_link_changes(
                handle,
                BridgeConf::gen_link_msg_builder(des_iface),
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(IfaceType::Veth) => {
            apply_base_link_changes(
                handle,
                VethConf::gen_link_msg_builder(des_iface, cur_iface)?,
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(IfaceType::Bond) => {
            apply_base_link_changes(
                handle,
                BondConf::gen_link_msg_builder(handle, des_iface, cur_iface)
                    .await?,
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(IfaceType::Vlan) => {
            apply_base_link_changes(
                handle,
                VlanConf::gen_link_msg_builder(handle, des_iface, cur_iface)
                    .await?,
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(IfaceType::Dummy) => {
            apply_base_link_changes(
                handle,
                DummyConf::gen_link_msg_builder(des_iface),
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(t) => {
            return Err(NisporError::invalid_argument(format!(
                "Unsupported interface type {t}: {des_iface:?}",
            )));
        }
        None => {
            apply_base_link_changes(
                handle,
                LinkUnspec::new_with_name(des_iface.name.as_str()),
                des_iface,
                cur_iface,
            )
            .await?
        }
    })
}

async fn delete_iface(
    handle: &rtnetlink::Handle,
    iface_name: &str,
    index: u32,
) -> Result<(), NisporError> {
    handle.link().del(index).execute().await.map_err(|e| {
        NisporError::new(
            ErrorKind::NisporBug,
            format!("Failed to delete interface {iface_name}: {e}"),
        )
    })
}

async fn send_change_netlink(
    handle: &rtnetlink::Handle,
    msg: LinkMessage,
    iface_name: &str,
) -> Result<(), NisporError> {
    log::trace!("Changing interface by netlink message {msg:?}");
    handle
        .link()
        .add(msg)
        // Even we are changing existing interface, kernel still require us to
        // use `RTM_NEWLINK`. The `RTM_SETLINK` is only used for bridge VLAN
        // filtering.
        //
        // TODO: Use `rtnetlink::LinkHandler::change()` once they
        // released.
        .set_flags(NLM_F_ACK | NLM_F_REQUEST)
        .execute()
        .await
        .map_err(|e| {
            NisporError::new(
                ErrorKind::NisporBug,
                format!("Failed to change interface {iface_name}: {e}"),
            )
        })
}
