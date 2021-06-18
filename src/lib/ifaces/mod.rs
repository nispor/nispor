// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod bond;
mod bridge;
mod ethtool;
mod iface;
mod mac_vlan;
mod mac_vtap;
mod sriov;
mod tun;
mod veth;
mod vlan;
mod vrf;
mod vxlan;

pub use crate::ifaces::bond::BondAdInfo;
pub use crate::ifaces::bond::BondAdSelect;
pub use crate::ifaces::bond::BondAllSubordinatesActive;
pub use crate::ifaces::bond::BondArpValidate;
pub use crate::ifaces::bond::BondFailOverMac;
pub use crate::ifaces::bond::BondInfo;
pub use crate::ifaces::bond::BondLacpRate;
pub use crate::ifaces::bond::BondMiiStatus;
pub use crate::ifaces::bond::BondMode;
pub use crate::ifaces::bond::BondModeArpAllTargets;
pub use crate::ifaces::bond::BondPrimaryReselect;
pub use crate::ifaces::bond::BondSubordinateInfo;
pub use crate::ifaces::bond::BondSubordinateState;
pub use crate::ifaces::bond::BondXmitHashPolicy;
pub use crate::ifaces::bridge::BridgeInfo;
pub use crate::ifaces::bridge::BridgePortInfo;
pub use crate::ifaces::bridge::BridgeVlanEntry;
pub use crate::ifaces::ethtool::{EthtoolInfo, EthtoolPauseInfo};
pub(crate) use crate::ifaces::iface::get_iface_name_by_index;
pub use crate::ifaces::iface::ControllerType;
pub use crate::ifaces::iface::Iface;
pub use crate::ifaces::iface::IfaceConf;
pub use crate::ifaces::iface::IfaceFlags;
pub use crate::ifaces::iface::IfaceState;
pub use crate::ifaces::iface::IfaceType;
pub use crate::ifaces::mac_vlan::MacVlanInfo;
pub use crate::ifaces::mac_vtap::MacVtapInfo;
pub use crate::ifaces::sriov::SriovInfo;
pub use crate::ifaces::tun::TunInfo;
pub use crate::ifaces::veth::VethInfo;
pub use crate::ifaces::vlan::VlanInfo;
pub use crate::ifaces::vlan::VlanProtocol;
pub use crate::ifaces::vrf::VrfInfo;
pub use crate::ifaces::vrf::VrfSubordinateInfo;
pub use crate::ifaces::vxlan::VxlanInfo;

use crate::ifaces::bond::bond_iface_tidy_up;
use crate::ifaces::bridge::bridge_iface_tidy_up;
use crate::ifaces::ethtool::get_ethtool_infos;
use crate::ifaces::iface::fill_bridge_vlan_info;
use crate::ifaces::iface::parse_nl_msg_to_iface;
use crate::ifaces::mac_vlan::mac_vlan_iface_tidy_up;
use crate::ifaces::veth::veth_iface_tidy_up;
use crate::ifaces::vlan::vlan_iface_tidy_up;
use crate::ifaces::vrf::vrf_iface_tidy_up;
use crate::ifaces::vxlan::vxlan_iface_tidy_up;
use crate::netlink::fill_ip_addr;
use crate::NisporError;
use futures::stream::TryStreamExt;
use netlink_packet_route::rtnl::constants::AF_BRIDGE;
use netlink_packet_route::rtnl::constants::AF_UNSPEC;
use netlink_packet_route::rtnl::constants::RTEXT_FILTER_BRVLAN_COMPRESSED;
use netlink_packet_route::rtnl::constants::RTEXT_FILTER_VF;
use rtnetlink::new_connection;
use std::collections::HashMap;

pub(crate) async fn get_ifaces() -> Result<HashMap<String, Iface>, NisporError>
{
    let mut iface_states: HashMap<String, Iface> = HashMap::new();
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);

    let mut links = handle
        .link()
        .get()
        .set_filter_mask(AF_UNSPEC as u8, RTEXT_FILTER_VF)
        .execute();
    while let Some(nl_msg) = links.try_next().await? {
        if let Some(iface_state) = parse_nl_msg_to_iface(&nl_msg)? {
            iface_states.insert(iface_state.name.clone(), iface_state);
        }
    }
    let mut addrs = handle.address().get().execute();
    while let Some(nl_msg) = addrs.try_next().await? {
        fill_ip_addr(&mut iface_states, &nl_msg)?;
    }
    let mut br_vlan_links = handle
        .link()
        .get()
        .set_filter_mask(AF_BRIDGE as u8, RTEXT_FILTER_BRVLAN_COMPRESSED)
        .execute();
    while let Some(nl_msg) = br_vlan_links.try_next().await? {
        fill_bridge_vlan_info(&mut iface_states, &nl_msg)?;
    }

    match get_ethtool_infos().await {
        Ok(mut ethtool_infos) => {
            ifaces_merge_ethool_infos(&mut iface_states, &mut ethtool_infos);
        }
        Err(e) => {
            // Ethtool is considered as optional
            eprintln!("Failed to query ethtool info: {}", e);
        }
    };

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
