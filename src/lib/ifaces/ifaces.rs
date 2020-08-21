use crate::ifaces::bond::bond_iface_tidy_up;
use crate::ifaces::bridge::bridge_iface_tidy_up;
use crate::ifaces::iface::fill_bridge_vlan_info;
use crate::ifaces::iface::parse_nl_msg_to_iface;
use crate::ifaces::veth::veth_iface_tidy_up;
use crate::ifaces::vlan::vlan_iface_tidy_up;
use crate::ifaces::vrf::vrf_iface_tidy_up;
use crate::ifaces::vxlan::vxlan_iface_tidy_up;
use crate::netlink::fill_ip_addr;
use crate::Iface;
use crate::NisporError;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::constants::AF_BRIDGE;
use netlink_packet_route::rtnl::constants::AF_UNSPEC;
use netlink_sys::constants::RTEXT_FILTER_BRVLAN_COMPRESSED;
use netlink_sys::constants::RTEXT_FILTER_VF;
use rtnetlink::new_connection;
use std::collections::HashMap;
use tokio::runtime::Runtime;

async fn _get_ifaces() -> Result<HashMap<String, Iface>, NisporError> {
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle
        .link()
        .get()
        .set_filter_mask(AF_UNSPEC as u8, RTEXT_FILTER_VF)
        .execute();
    while let Some(nl_msg) = links.try_next().await? {
        if let Some(iface_state) = parse_nl_msg_to_iface(&nl_msg) {
            iface_states.insert(iface_state.name.clone(), iface_state);
        }
    }
    let mut addrs = handle.address().get().execute();
    while let Some(nl_msg) = addrs.try_next().await? {
        fill_ip_addr(&mut iface_states, &nl_msg);
    }
    let mut br_vlan_links = handle
        .link()
        .get()
        .set_filter_mask(AF_BRIDGE as u8, RTEXT_FILTER_BRVLAN_COMPRESSED)
        .execute();
    while let Some(nl_msg) = br_vlan_links.try_next().await? {
        fill_bridge_vlan_info(&mut iface_states, &nl_msg);
    }

    tidy_up(&mut iface_states);
    Ok(iface_states)
}

pub(crate) fn get_ifaces() -> Result<HashMap<String, Iface>, NisporError> {
    Ok(Runtime::new()?.block_on(_get_ifaces())?)
}

fn tidy_up(iface_states: &mut HashMap<String, Iface>) {
    controller_iface_index_to_name(iface_states);
    bond_iface_tidy_up(iface_states);
    bridge_iface_tidy_up(iface_states);
    vlan_iface_tidy_up(iface_states);
    vxlan_iface_tidy_up(iface_states);
    veth_iface_tidy_up(iface_states);
    vrf_iface_tidy_up(iface_states);
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
