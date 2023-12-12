// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{self, LinkVfInfo};

use serde::{Deserialize, Serialize};

use crate::{
    mac::{parse_as_mac, ETH_ALEN, INFINIBAND_ALEN},
    Iface, IfaceType, NisporError, VlanProtocol,
};

const MAX_ADDR_LEN: usize = 32;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum VfLinkState {
    #[default]
    Auto,
    Enable,
    Disable,
    Other(u32),
    Unknown,
}

impl From<link::VfLinkState> for VfLinkState {
    fn from(d: link::VfLinkState) -> Self {
        match d {
            link::VfLinkState::Auto => VfLinkState::Auto,
            link::VfLinkState::Enable => VfLinkState::Enable,
            link::VfLinkState::Disable => VfLinkState::Disable,
            _ => VfLinkState::Other(d.into()),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Default)]
#[non_exhaustive]
pub struct VfState {
    pub rx_packets: u64,
    pub tx_packets: u64,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub broadcast: u64,
    pub multicast: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct SriovInfo {
    pub vfs: Vec<VfInfo>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VfInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iface_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pf_name: Option<String>,
    pub id: u32,
    pub mac: String,
    pub broadcast: String,
    // 0 disables VLAN filter
    pub vlan_id: u32,
    pub qos: u32,
    pub vlan_proto: VlanProtocol,
    // Max TX bandwidth in Mbps, 0 disables throttling
    pub tx_rate: u32,
    pub spoof_check: bool,
    pub link_state: VfLinkState,
    // Min Bandwidth in Mbps
    pub min_tx_rate: u32,
    // Max Bandwidth in Mbps
    pub max_tx_rate: u32,
    pub query_rss: bool,
    pub state: VfState,
    pub trust: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ib_node_guid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ib_port_guid: Option<String>,
}

pub(crate) fn get_sriov_info(
    pf_iface_name: &str,
    nlas: &[LinkVfInfo],
    iface_type: &IfaceType,
) -> Result<SriovInfo, NisporError> {
    let mut sriov_info = SriovInfo::default();
    let mac_len = match iface_type {
        IfaceType::Ethernet => ETH_ALEN,
        IfaceType::Infiniband => INFINIBAND_ALEN,
        _ => MAX_ADDR_LEN,
    };
    for port_nlas in nlas {
        let mut vf_info = VfInfo::default();
        for port_info in &port_nlas.0 {
            match port_info {
                link::VfInfo::Mac(m) => {
                    vf_info.id = m.vf_id;
                    vf_info.iface_name =
                        get_vf_iface_name(pf_iface_name, &vf_info.id);
                    vf_info.pf_name = Some(pf_iface_name.to_string());
                    vf_info.mac = parse_as_mac(mac_len, &m.mac)?;
                }
                link::VfInfo::Vlan(v) => {
                    vf_info.vlan_id = v.vlan_id;
                    vf_info.qos = v.qos;
                }
                link::VfInfo::TxRate(v) => vf_info.tx_rate = v.rate,
                link::VfInfo::SpoofCheck(v) => vf_info.spoof_check = v.enabled,
                link::VfInfo::LinkState(v) => {
                    vf_info.link_state = v.state.into();
                }
                link::VfInfo::Rate(v) => {
                    vf_info.min_tx_rate = v.min_tx_rate;
                    vf_info.max_tx_rate = v.max_tx_rate;
                }
                link::VfInfo::RssQueryEn(v) => vf_info.query_rss = v.enabled,
                link::VfInfo::Stats(v) => {
                    vf_info.state = parse_vf_stats(v.as_slice())?;
                }
                link::VfInfo::Trust(v) => vf_info.trust = v.enabled,
                link::VfInfo::IbNodeGuid(v) => {
                    vf_info.ib_node_guid = Some(format!("{:X}", v.guid));
                }
                link::VfInfo::IbPortGuid(v) => {
                    vf_info.ib_port_guid = Some(format!("{:X}", v.guid));
                }
                link::VfInfo::VlanList(v) => {
                    if let Some(link::VfVlan::Info(vf_vlan_info)) = v.first() {
                        vf_info.vlan_proto = vf_vlan_info.protocol.into();
                    }
                }
                link::VfInfo::Broadcast(v) => {
                    vf_info.broadcast = parse_as_mac(mac_len, &v.addr)?;
                }
                _ => {
                    log::debug!("Unhandled SRIOV NLA {port_info:?}",);
                }
            }
        }

        sriov_info.vfs.push(vf_info);
    }
    Ok(sriov_info)
}

fn parse_vf_stats(nlas: &[link::VfStats]) -> Result<VfState, NisporError> {
    let mut state = VfState::default();
    for nla in nlas {
        match nla {
            link::VfStats::RxPackets(d) => state.rx_packets = *d,
            link::VfStats::TxPackets(d) => state.tx_packets = *d,
            link::VfStats::RxBytes(d) => state.rx_bytes = *d,
            link::VfStats::TxBytes(d) => state.tx_bytes = *d,
            link::VfStats::Broadcast(d) => state.broadcast = *d,
            link::VfStats::Multicast(d) => state.multicast = *d,
            link::VfStats::RxDropped(d) => state.rx_dropped = *d,
            link::VfStats::TxDropped(d) => state.tx_dropped = *d,
            _ => log::debug!("Unhandled IFLA_VF_STATS {:?}", nla),
        }
    }
    Ok(state)
}

// Currently there is no valid netlink way to get information as the kernel code
// is in at PCI level: drivers/pci/iov.c
// We use sysfs content /sys/class/net/<pf_name>/devices/virtfn<sriov_id>/net/
fn get_vf_iface_name(pf_name: &str, sriov_id: &u32) -> Option<String> {
    let sysfs_path =
        format!("/sys/class/net/{pf_name}/device/virtfn{sriov_id}/net/");
    read_folder(&sysfs_path).pop()
}

fn read_folder(folder_path: &str) -> Vec<String> {
    let mut folder_contents = Vec::new();
    let fd = match std::fs::read_dir(folder_path) {
        Ok(f) => f,
        Err(e) => {
            log::warn!("Failed to read dir {}: {}", folder_path, e);
            return folder_contents;
        }
    };
    for entry in fd {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read dir {}: {}", folder_path, e);
                continue;
            }
        };
        let path = entry.path();
        if let Ok(content) = path.strip_prefix(folder_path) {
            if let Some(content_str) = content.to_str() {
                folder_contents.push(content_str.to_string());
            }
        }
    }
    folder_contents
}

// Fill the VfInfo base PF state
pub(crate) fn sriov_vf_iface_tidy_up(
    iface_states: &mut HashMap<String, Iface>,
) {
    let mut vf_info_dict: HashMap<String, VfInfo> = HashMap::new();

    for iface in iface_states.values() {
        if let Some(sriov_conf) = iface.sriov.as_ref() {
            for vf_info in sriov_conf.vfs.as_slice() {
                if let Some(vf_name) = vf_info.iface_name.as_ref() {
                    vf_info_dict.insert(vf_name.to_string(), vf_info.clone());
                }
            }
        }
    }
    for (vf_name, vf_info) in vf_info_dict.drain() {
        if let Some(vf_iface) = iface_states.get_mut(vf_name.as_str()) {
            vf_iface.sriov_vf = Some(vf_info);
        }
    }
}
