// SPDX-License-Identifier: Apache-2.0

use crate::BridgeVlanEntry;
use crate::NisporError;
use netlink_packet_route::rtnl::link::nlas::{AfSpecBridge, BridgeVlanInfo};

// VLAN is PVID, ingress untagged;
const BRIDGE_VLAN_INFO_PVID: u16 = 1 << 1;
// VLAN egresses untagged;
const BRIDGE_VLAN_INFO_UNTAGGED: u16 = 1 << 2;
// VLAN is start of vlan range;
const BRIDGE_VLAN_INFO_RANGE_BEGIN: u16 = 1 << 3;
// VLAN is end of vlan range;
const BRIDGE_VLAN_INFO_RANGE_END: u16 = 1 << 4;

// TODO: Dup with parse_bond_info
pub(crate) fn parse_af_spec_bridge_info(
    nlas: &[AfSpecBridge],
) -> Result<Option<Vec<BridgeVlanEntry>>, NisporError> {
    let mut vlans = Vec::new();

    for nla in nlas {
        if let AfSpecBridge::VlanInfo(nla_vlan_info) = nla {
            if let Some(v) = parse_vlan_info(nla_vlan_info)? {
                vlans.push(v);
            }
        }
    }

    if !vlans.is_empty() {
        Ok(Some(merge_vlan_range(&vlans)))
    } else {
        Ok(None)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
struct KernelBridgeVlanEntry {
    vid: u16,
    is_pvid: bool, // is PVID and ingress untagged
    is_egress_untagged: bool,
    is_range_start: bool,
    is_range_end: bool,
}

fn parse_vlan_info(
    nla: &BridgeVlanInfo,
) -> Result<Option<KernelBridgeVlanEntry>, NisporError> {
    let mut entry = KernelBridgeVlanEntry {
        vid: nla.vid,
        ..Default::default()
    };
    entry.is_pvid = (nla.flags & BRIDGE_VLAN_INFO_PVID) > 0;
    entry.is_egress_untagged = (nla.flags & BRIDGE_VLAN_INFO_UNTAGGED) > 0;
    entry.is_range_start = (nla.flags & BRIDGE_VLAN_INFO_RANGE_BEGIN) > 0;
    entry.is_range_end = (nla.flags & BRIDGE_VLAN_INFO_RANGE_END) > 0;
    Ok(Some(entry))
}

fn merge_vlan_range(
    kernel_vlans: &[KernelBridgeVlanEntry],
) -> Vec<BridgeVlanEntry> {
    let mut vlans = Vec::new();
    let mut vlan_start = None;
    for k_vlan in kernel_vlans {
        match (k_vlan.is_range_start, k_vlan.is_range_end) {
            (true, false) => {
                vlan_start = Some(k_vlan.vid);
                continue;
            }
            (false, true) => {
                if let Some(start) = vlan_start {
                    vlans.push(BridgeVlanEntry {
                        vid: None,
                        vid_range: Some((start, k_vlan.vid)),
                        is_pvid: k_vlan.is_pvid,
                        is_egress_untagged: k_vlan.is_egress_untagged,
                    })
                } else {
                    log::warn!(
                        "Invalid kernel bridge vlan information: \
                        missing start VLAN for {}",
                        k_vlan.vid
                    );
                }
                vlan_start = None;
            }
            (false, false) | (true, true) => {
                vlans.push(BridgeVlanEntry {
                    vid: Some(k_vlan.vid),
                    vid_range: None,
                    is_pvid: k_vlan.is_pvid,
                    is_egress_untagged: k_vlan.is_egress_untagged,
                });
                vlan_start = None;
            }
        };
    }
    vlans
}
