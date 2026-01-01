// SPDX-License-Identifier: Apache-2.0

use rtnetlink::packet_route::link::{
    AfSpecBridge, BridgeVlanInfo, BridgeVlanInfoFlags,
};
use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceType, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(deny_unknown_fields)]
pub struct BridgeVlanEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid_range: Option<(u16, u16)>,
    #[serde(default)]
    pub is_pvid: bool, // is PVID and ingress untagged
    #[serde(default)]
    pub is_egress_untagged: bool,
    /// Only for apply action
    #[serde(default, skip_serializing)]
    pub remove: bool,
}

pub(crate) fn parse_bridge_vlan_info(
    iface_state: &mut Iface,
    nlas: &[AfSpecBridge],
) -> Result<(), NisporError> {
    if let Some(ref mut port_info) = iface_state.bridge_port {
        if let Some(cur_vlans) = parse_af_spec_bridge_info(nlas)? {
            match port_info.vlans.as_mut() {
                Some(vlans) => vlans.extend(cur_vlans),
                None => port_info.vlans = Some(cur_vlans),
            };
        }
    } else if iface_state.iface_type == IfaceType::Bridge {
        let br_vlan = iface_state.bridge_vlan.get_or_insert(Vec::new());
        // It's the VLAN of the bridge itself
        if let Some(cur_vlans) = parse_af_spec_bridge_info(nlas)? {
            br_vlan.extend(cur_vlans);
        }
    }
    Ok(())
}

fn parse_af_spec_bridge_info(
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
    entry.is_pvid = nla.flags.contains(BridgeVlanInfoFlags::Pvid);
    entry.is_egress_untagged =
        nla.flags.contains(BridgeVlanInfoFlags::Untagged);
    entry.is_range_start = nla.flags.contains(BridgeVlanInfoFlags::RangeBegin);
    entry.is_range_end = nla.flags.contains(BridgeVlanInfoFlags::RangeEnd);
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
                        ..Default::default()
                    })
                } else {
                    log::warn!(
                        "Invalid kernel bridge vlan information: missing \
                         start VLAN for {}",
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
                    ..Default::default()
                });
                vlan_start = None;
            }
        };
    }
    vlans
}
