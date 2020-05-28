use crate::netlink::nla::parse_nla_header;
use crate::netlink::nla::NL_ATTR_HDR_LEN;
use crate::BridgeVlanEntry;
use std::convert::TryInto;

const IFLA_BRIDGE_VLAN_INFO: u16 = 2;

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
    raw: &[u8],
) -> Option<Vec<BridgeVlanEntry>> {
    let mut i: usize = 0;
    let mut vlans = Vec::new();

    // TODO: Dup with parse_bond_info
    while i < raw.len() {
        let hdr_ptr = raw.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = raw
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            std::slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        match hdr.nla_type {
            IFLA_BRIDGE_VLAN_INFO => {
                if let Some(v) = parse_vlan_info(data) {
                    vlans.push(v);
                }
            }
            _ => {
                eprintln!(
                    "unknown nla_type, {}, nla_data: {:?}",
                    hdr.nla_type, data
                );
            }
        }
        i = i + hdr.nla_len;
    }
    if vlans.len() > 0 {
        Some(merge_vlan_range(&vlans))
    } else {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
struct KernelBridgeVlanEntry {
    vid: u16,
    is_pvid: bool, // is PVID and ingress untagged
    is_egress_untagged: bool,
    is_range_start: bool,
    is_range_end: bool,
}

fn parse_vlan_info(data: &[u8]) -> Option<KernelBridgeVlanEntry> {
    if data.len() == 4 {
        let flags = u16::from_ne_bytes([data[0], data[1]]);
        let vid = u16::from_ne_bytes([data[2], data[3]]);
        let mut entry = KernelBridgeVlanEntry {
            vid: vid,
            ..Default::default()
        };
        entry.is_pvid = (flags & BRIDGE_VLAN_INFO_PVID) > 0;
        entry.is_egress_untagged = (flags & BRIDGE_VLAN_INFO_UNTAGGED) > 0;
        entry.is_range_start = (flags & BRIDGE_VLAN_INFO_RANGE_BEGIN) > 0;
        entry.is_range_end = (flags & BRIDGE_VLAN_INFO_RANGE_END) > 0;
        Some(entry)
    } else {
        eprintln!(
            "Invalid kernel bridge vlan info: {:?}, should be [u8;4]",
            data
        );
        None
    }
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
                    eprintln!(
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
