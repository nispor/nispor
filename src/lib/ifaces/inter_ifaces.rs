// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::{
    constants::AF_BRIDGE, AF_UNSPEC, RTEXT_FILTER_BRVLAN_COMPRESSED,
    RTEXT_FILTER_VF,
};
use rtnetlink::new_connection;

use super::{
    bond::bond_iface_tidy_up,
    bridge::bridge_iface_tidy_up,
    ethtool::get_ethtool_infos,
    fill_bridge_vlan_info,
    iface::{change_iface_mac, change_iface_state},
    ipoib::ipoib_iface_tidy_up,
    mac_vlan::mac_vlan_iface_tidy_up,
    macsec::macsec_iface_tidy_up,
    parse_nl_msg_to_iface, parse_nl_msg_to_name_and_index,
    sriov::sriov_vf_iface_tidy_up,
    veth::veth_iface_tidy_up,
    vlan::vlan_iface_tidy_up,
    vrf::vrf_iface_tidy_up,
    vxlan::vxlan_iface_tidy_up,
};
use crate::{
    ip::change_ips, netlink::fill_ip_addr, BondConf, BridgeConf, EthtoolInfo,
    Iface, IfaceConf, IfaceState, IfaceType, NetStateIfaceFilter, NisporError,
    VlanConf,
};

pub(crate) async fn get_ifaces(
    filter: Option<&NetStateIfaceFilter>,
) -> Result<HashMap<String, Iface>, NisporError> {
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection()?;

    tokio::spawn(connection);

    let default_filter = NetStateIfaceFilter::default();

    let filter = filter.unwrap_or(&default_filter);

    let mut link_get_handle = handle.link().get();

    if filter.include_sriov_vf_info {
        link_get_handle =
            link_get_handle.set_filter_mask(AF_UNSPEC as u8, RTEXT_FILTER_VF);
    }
    if let Some(iface_name) = filter.iface_name.as_ref() {
        link_get_handle = link_get_handle.match_name(iface_name.to_string());
    }

    let mut links = link_get_handle.execute();
    while let Some(nl_msg) = links.try_next().await? {
        if let Some(iface_state) = parse_nl_msg_to_iface(&nl_msg)? {
            iface_states.insert(iface_state.name.clone(), iface_state);
        }
    }

    let iface_index = filter
        .iface_name
        .as_ref()
        .and_then(|name| iface_states.get(name))
        .map(|i| i.index);

    if filter.iface_name.is_some() && iface_index.is_none() {
        return Err(NisporError::invalid_argument(format!(
            "Interface {} not found",
            filter.iface_name.as_ref().unwrap()
        )));
    }

    if filter.include_ip_address || filter.include_mptcp {
        let mut addr_get_handle = handle.address().get();
        if let Some(iface_index) = iface_index {
            // rust-rtnetlink is doing filter this at userspace level.
            // https://github.com/little-dude/netlink/issues/294
            addr_get_handle =
                addr_get_handle.set_link_index_filter(iface_index);
        }

        let mut addrs = addr_get_handle.execute();
        while let Some(nl_msg) = addrs.try_next().await? {
            fill_ip_addr(&mut iface_states, &nl_msg)?;
        }
    }

    if filter.include_bridge_vlan {
        let mut link_get_handle = handle
            .link()
            .get()
            .set_filter_mask(AF_BRIDGE as u8, RTEXT_FILTER_BRVLAN_COMPRESSED);

        if let Some(iface_name) = filter.iface_name.as_ref() {
            link_get_handle =
                link_get_handle.match_name(iface_name.to_string());
        }

        let mut br_vlan_links = link_get_handle.execute();
        while let Some(nl_msg) = br_vlan_links.try_next().await? {
            fill_bridge_vlan_info(&mut iface_states, &nl_msg)?;
        }
    }

    if filter.include_ethtool {
        // TODO: Apply interface filter to ethtool dump also
        match get_ethtool_infos().await {
            Ok(mut ethtool_infos) => {
                ifaces_merge_ethool_infos(
                    &mut iface_states,
                    &mut ethtool_infos,
                );
            }
            Err(e) => {
                // Ethtool is considered as optional
                log::warn!("Failed to query ethtool info: {}", e);
            }
        };
    }

    tidy_up(&mut iface_states);
    Ok(iface_states)
}

fn tidy_up(iface_states: &mut HashMap<String, Iface>) {
    controller_iface_index_to_name(iface_states);
    bond_iface_tidy_up(iface_states);
    bridge_iface_tidy_up(iface_states);
    vlan_iface_tidy_up(iface_states);
    vxlan_iface_tidy_up(iface_states);
    veth_iface_tidy_up(iface_states);
    vrf_iface_tidy_up(iface_states);
    mac_vlan_iface_tidy_up(iface_states);
    macsec_iface_tidy_up(iface_states);
    ipoib_iface_tidy_up(iface_states);
    sriov_vf_iface_tidy_up(iface_states);
}

fn controller_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if let Some(controller) = &iface.controller {
            if let Some(name) = index_to_name.get(controller) {
                iface.controller = Some(name.to_string());
            }
        }
    }
}

fn ifaces_merge_ethool_infos(
    iface_states: &mut HashMap<String, Iface>,
    ethtool_infos: &mut HashMap<String, EthtoolInfo>,
) {
    for iface in iface_states.values_mut() {
        if let Some(ethtool_info) = ethtool_infos.remove(&iface.name) {
            iface.ethtool = Some(ethtool_info)
        }
    }
}

pub(crate) async fn get_iface_name2index(
) -> Result<HashMap<String, u32>, NisporError> {
    let mut name2index: HashMap<String, u32> = HashMap::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle.link().get().execute();
    while let Some(nl_msg) = links.try_next().await? {
        if let Some((iface_name, iface_index)) =
            parse_nl_msg_to_name_and_index(&nl_msg)
        {
            name2index.insert(iface_name, iface_index);
        }
    }
    Ok(name2index)
}

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
                BondConf::create(&handle, &iface.name).await?;
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
                                .master(ctrl_iface.index)
                                .execute()
                                .await?;
                        }
                    },
                    None => {
                        handle
                            .link()
                            .set(cur_iface.index)
                            .nomaster()
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
