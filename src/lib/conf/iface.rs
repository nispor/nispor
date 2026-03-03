// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    LinkMessageBuilder, LinkUnspec,
    packet_route::link::{InfoKind, LinkMessage},
};
use serde::{Deserialize, Serialize};

use super::{
    super::query::get_ifaces_with_handle, alt_name::change_iface_alt_name,
    base_iface::apply_base_link_changes, ip::change_ip_layer,
};
use crate::{
    AltNameConf, BondConf, BondPortConf, BridgeConf, BridgePortConf, DummyConf,
    ErrorKind, Iface, IfaceState, IfaceType, IpConf, NetStateIfaceFilter,
    NisporError, VethConf, VlanConf, WireguardConf,
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
    pub bond_port: Option<BondPortConf>,
    pub bridge_port: Option<BridgePortConf>,
    pub wireguard: Option<WireguardConf>,
}

impl IfaceConf {
    /// Need interfaces to be down state for these changes:
    ///  * Change MAC address
    ///  * Change controller
    ///  * Change bond mode
    pub(crate) fn need_state_down_before_apply(&self, current: &Iface) -> bool {
        if let Some(des_mac) = self.mac_address.as_ref()
            && des_mac.to_uppercase() != current.mac_address.to_uppercase()
        {
            return true;
        }

        if self.controller.is_some() && self.controller != current.controller {
            return true;
        }

        if let Some(des_bond_mode) = self.bond.as_ref().and_then(|b| b.mode)
            && let Some(cur_bond_mode) = current.bond.as_ref().map(|b| b.mode)
            && des_bond_mode != cur_bond_mode
        {
            return true;
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
) -> Result<(), NisporError> {
    // Instead of query full current state, we query desired interface only
    // one by one, because:
    // 1. Consistent performance regardless current interface amount.
    // 2. Allowing removing interface and add it back sequentially.
    let cur_iface = get_cur_iface(handle, des_iface).await;
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
        let mut msgs =
            gen_link_msg(handle, des_iface, cur_iface.as_ref()).await?;
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
                        ErrorKind::Bug,
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

        // Refresh current interface for newly created interface or
        // controller/port attached
        if let Some(cur_iface) = get_cur_iface(handle, des_iface).await {
            change_port_config(handle, des_iface, &cur_iface).await?;
            change_iface_alt_name(handle, des_iface, &cur_iface).await?;
            change_ip_layer(handle, des_iface, &cur_iface).await?;
        }
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
        Some(IfaceType::Dummy) if cur_iface.is_none() => {
            apply_base_link_changes(
                handle,
                DummyConf::gen_link_msg_builder(des_iface),
                des_iface,
                cur_iface,
            )
            .await?
        }
        Some(IfaceType::Wireguard) if cur_iface.is_none() => {
            apply_base_link_changes(
                handle,
                LinkMessageBuilder::<LinkUnspec>::new_with_info_kind(
                    InfoKind::Wireguard,
                )
                .name(des_iface.name.clone()),
                des_iface,
                cur_iface,
            )
            .await?
        }
        _ => {
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
            ErrorKind::Bug,
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
    handle.link().change(msg).execute().await.map_err(|e| {
        NisporError::new(
            ErrorKind::Bug,
            format!("Failed to change interface {iface_name}: {e}"),
        )
    })
}

async fn get_cur_iface(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
) -> Option<Iface> {
    let mut iface_filter = NetStateIfaceFilter::minimum();
    iface_filter.iface_name = Some(des_iface.name.to_string());
    if des_iface.ipv4.is_some() || des_iface.ipv6.is_some() {
        iface_filter.include_ip_address = true;
    }
    if let Ok(mut cur_ifaces) =
        get_ifaces_with_handle(handle, Some(&iface_filter)).await
    {
        cur_ifaces.remove(&des_iface.name)
    } else {
        None
    }
}

async fn change_port_config(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: &Iface,
) -> Result<(), NisporError> {
    if let Some(bond_port_conf) = des_iface.bond_port.as_ref() {
        send_change_netlink(
            handle,
            bond_port_conf.gen_link_msg(cur_iface),
            des_iface.name.as_str(),
        )
        .await?;
    }
    if let Some(bridge_port_conf) = des_iface.bridge_port.as_ref() {
        send_change_netlink(
            handle,
            bridge_port_conf.gen_port_conf_link_msg(cur_iface),
            des_iface.name.as_str(),
        )
        .await?;

        apply_bridge_vlan_conf(
            handle,
            des_iface.name.as_str(),
            bridge_port_conf.gen_del_port_vlan_conf_link_msg(cur_iface),
            bridge_port_conf.gen_add_port_vlan_conf_link_msg(cur_iface),
            "port",
        )
        .await?;
    }
    if let Some(br_conf) = des_iface.bridge.as_ref() {
        apply_bridge_vlan_conf(
            handle,
            des_iface.name.as_str(),
            br_conf.gen_del_vlan_conf_link_msg(cur_iface),
            br_conf.gen_add_vlan_conf_link_msg(cur_iface),
            "self",
        )
        .await?;
    }
    Ok(())
}

async fn apply_bridge_vlan_conf(
    handle: &rtnetlink::Handle,
    iface_name: &str,
    del_msg: Option<LinkMessage>,
    add_msg: Option<LinkMessage>,
    vlan_entity: &str,
) -> Result<(), NisporError> {
    if let Some(msg) = del_msg {
        log::trace!(
            "Remove bridge {} VLAN via netlink message {msg:?}",
            vlan_entity
        );
        handle
            .link()
            .del_with_message(msg)
            .execute()
            .await
            .map_err(|e| {
                NisporError::new(
                    ErrorKind::Bug,
                    format!(
                        "Failed to remove bridge {} vlan of interface {}: {e}",
                        vlan_entity, iface_name
                    ),
                )
            })?;
    }
    if let Some(msg) = add_msg {
        log::trace!(
            "Set bridge {} VLAN via netlink message {msg:?}",
            vlan_entity
        );
        handle.link().set(msg).execute().await.map_err(|e| {
            NisporError::new(
                ErrorKind::Bug,
                format!(
                    "Failed to set bridge {} vlan of interface {}: {e}",
                    vlan_entity, iface_name
                ),
            )
        })?;
    }
    Ok(())
}
