use crate::ifaces::bond::bond_iface_tidy_up;
use crate::ifaces::bridge::bridge_iface_tidy_up;
use crate::ifaces::iface::fill_bridge_vlan_info;
use crate::ifaces::iface::parse_nl_msg_to_iface;
use crate::netlink::fill_ip_addr;
use crate::Iface;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::constants::AF_BRIDGE;
use netlink_sys::constants::RTEXT_FILTER_BRVLAN_COMPRESSED;
use rtnetlink::{new_connection, Error};
use std::collections::HashMap;
use tokio::runtime::Runtime;

async fn _get_ifaces() -> Result<HashMap<String, Iface>, Error> {
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    let mut links = handle.link().get().execute();
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

pub(crate) fn get_ifaces() -> HashMap<String, Iface> {
    Runtime::new().unwrap().block_on(_get_ifaces()).unwrap()
}

fn tidy_up(iface_states: &mut HashMap<String, Iface>) {
    master_iface_index_to_name(iface_states);
    bond_iface_tidy_up(iface_states);
    bridge_iface_tidy_up(iface_states);
}

fn master_iface_index_to_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if let Some(master) = &iface.master {
            if let Some(name) = index_to_name.get(master) {
                iface.master = Some(name.to_string());
            }
        }
    }
}
