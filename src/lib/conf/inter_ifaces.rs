// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use rtnetlink::new_connection;

use super::{
    iface::{change_iface_mac, change_iface_state},
    ip::change_ips,
};
use crate::{
    BondConf, BridgeConf, Iface, IfaceConf, IfaceState, IfaceType, NisporError,
    VlanConf,
};

pub(crate) async fn delete_ifaces(
    ifaces: &[(&str, u32)],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for (iface_name, iface_index) in ifaces {
        if let Err(e) = handle.link().del(*iface_index).execute().await {
            return Err(NisporError::bug(format!(
                "Failed to delete interface {iface_name} with index {iface_index}: {e}"
            )));
        }
    }

    Ok(())
}

pub(crate) async fn create_ifaces(
    ifaces: &[&IfaceConf],
    cur_iface_name_2_index: &HashMap<String, u32>,
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for iface in ifaces {
        match iface.iface_type {
            Some(IfaceType::Bridge) => {
                BridgeConf::create(&handle, &iface.name).await?;
            }
            Some(IfaceType::Veth) => {
                if let Some(veth_conf) = &iface.veth {
                    veth_conf.create(&handle, &iface.name).await?;
                }
            }
            Some(IfaceType::Bond) => {
                if let Some(bond_conf) = iface.bond.as_ref() {
                    bond_conf.create(&handle, &iface.name).await?;
                } else {
                    BondConf::default().create(&handle, &iface.name).await?;
                }
            }
            Some(IfaceType::Vlan) => {
                if let Some(vlan_conf) = &iface.vlan {
                    if let Some(base_iface_index) =
                        cur_iface_name_2_index.get(&vlan_conf.base_iface)
                    {
                        VlanConf::create(
                            &handle,
                            &iface.name,
                            vlan_conf.vlan_id,
                            *base_iface_index,
                        )
                        .await?;
                    } else {
                        return Err(NisporError::invalid_argument(format!(
                            "Base interface {} for VLAN {} not found",
                            &vlan_conf.base_iface, iface.name
                        )));
                    }
                }
            }
            Some(_) => {
                return Err(NisporError::invalid_argument(format!(
                    "Cannot create unsupported interface {:?}",
                    &iface
                )));
            }
            None => {
                return Err(NisporError::invalid_argument(format!(
                    "No interface type defined for new interface {:?}",
                    &iface
                )));
            }
        }
    }

    Ok(())
}

pub(crate) async fn change_ifaces(
    ifaces: &[&IfaceConf],
    cur_ifaces: &HashMap<String, Iface>,
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    change_ifaces_mac(&handle, ifaces, cur_ifaces).await?;
    change_ifaces_controller(&handle, ifaces, cur_ifaces).await?;
    change_ifaces_state(&handle, ifaces, cur_ifaces).await?;
    change_ips(&handle, ifaces, cur_ifaces).await?;
    Ok(())
}

async fn change_ifaces_state(
    handle: &rtnetlink::Handle,
    ifaces: &[&IfaceConf],
    cur_ifaces: &HashMap<String, Iface>,
) -> Result<(), NisporError> {
    for iface in ifaces {
        if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
            if cur_iface.state != iface.state {
                if iface.state == IfaceState::Up {
                    change_iface_state(handle, cur_iface.index, true).await?;
                } else if iface.state == IfaceState::Down {
                    change_iface_state(handle, cur_iface.index, false).await?;
                } else {
                    return Err(NisporError::invalid_argument(format!(
                        "Unsupported interface state in NetConf: {}",
                        iface.state
                    )));
                }
            }
        }
    }

    Ok(())
}

async fn change_ifaces_controller(
    handle: &rtnetlink::Handle,
    ifaces: &[&IfaceConf],
    cur_ifaces: &HashMap<String, Iface>,
) -> Result<(), NisporError> {
    for iface in ifaces {
        if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
            if cur_iface.controller != iface.controller {
                match &iface.controller {
                    Some(ref ctrl_name) => match cur_ifaces.get(ctrl_name) {
                        None => {
                            return Err(NisporError::invalid_argument(
                                format!(
                                    "Controller interface {} not found",
                                    &ctrl_name
                                ),
                            ));
                        }
                        Some(ctrl_iface) => {
                            handle
                                .link()
                                .set(cur_iface.index)
                                .controller(ctrl_iface.index)
                                .execute()
                                .await?;
                        }
                    },
                    None => {
                        handle
                            .link()
                            .set(cur_iface.index)
                            .nocontroller()
                            .execute()
                            .await?;
                    }
                }
            }
        } else {
            return Err(NisporError::invalid_argument(format!(
                "Interface {} not found",
                iface.name
            )));
        }
    }

    Ok(())
}

async fn change_ifaces_mac(
    handle: &rtnetlink::Handle,
    ifaces: &[&IfaceConf],
    cur_ifaces: &HashMap<String, Iface>,
) -> Result<(), NisporError> {
    for iface in ifaces {
        if let Some(mac_addr) = &iface.mac_address {
            if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
                if cur_iface.state != IfaceState::Down {
                    // We can only change MAC address when link down
                    change_iface_state(handle, cur_iface.index, false).await?;
                }
                change_iface_mac(handle, cur_iface.index, mac_addr).await?;
                if cur_iface.state == IfaceState::Up
                    && iface.state == IfaceState::Up
                {
                    // Restore the interface state
                    change_iface_state(handle, cur_iface.index, true).await?;
                }
            }
        }
    }
    Ok(())
}
