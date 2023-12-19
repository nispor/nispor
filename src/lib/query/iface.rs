// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use netlink_packet_route::link::{
    self, InfoKind, InfoPortData, InfoPortKind, LinkAttribute, LinkInfo,
    LinkLayerType, LinkMessage,
};
use serde::{Deserialize, Serialize};

use super::{
    super::mac::parse_as_mac,
    bond::{get_bond_info, get_bond_subordinate_info},
    bridge::{get_bridge_info, get_bridge_port_info, parse_bridge_vlan_info},
    hsr::get_hsr_info,
    ip::fill_af_spec_inet_info,
    ipoib::get_ipoib_info,
    mac_vlan::get_mac_vlan_info,
    mac_vtap::get_mac_vtap_info,
    macsec::get_macsec_info,
    sriov::get_sriov_info,
    tun::get_tun_info,
    vlan::get_vlan_info,
    vrf::{get_vrf_info, get_vrf_subordinate_info},
    vxlan::get_vxlan_info,
    xfrm::get_xfrm_info,
};
use crate::{
    BondInfo, BondSubordinateInfo, BridgeInfo, BridgePortInfo, BridgeVlanEntry,
    EthtoolInfo, HsrInfo, IpoibInfo, Ipv4Info, Ipv6Info, MacSecInfo,
    MacVlanInfo, MacVtapInfo, MptcpAddress, NisporError, SriovInfo, TunInfo,
    VethInfo, VfInfo, VlanInfo, VrfInfo, VrfSubordinateInfo, VxlanInfo,
    XfrmInfo,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IfaceType {
    Bond,
    Veth,
    Bridge,
    Vlan,
    Dummy,
    Vxlan,
    Loopback,
    Ethernet,
    Infiniband,
    Vrf,
    Tun,
    MacVlan,
    MacVtap,
    OpenvSwitch,
    Ipoib,
    MacSec,
    Hsr,
    Unknown,
    Xfrm,
    Other(String),
}

impl Default for IfaceType {
    fn default() -> Self {
        IfaceType::Unknown
    }
}

impl std::fmt::Display for IfaceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Bond => "bond",
                Self::Veth => "veth",
                Self::Bridge => "bridge",
                Self::Vlan => "vlan",
                Self::Dummy => "dummy",
                Self::Vxlan => "vxlan",
                Self::Loopback => "loopback",
                Self::Ethernet => "ethernet",
                Self::Infiniband => "infiniband",
                Self::Vrf => "vrf",
                Self::Tun => "tun",
                Self::MacVlan => "macvlan",
                Self::MacVtap => "macvtap",
                Self::OpenvSwitch => "openvswitch",
                Self::Ipoib => "ipoib",
                Self::MacSec => "macsec",
                Self::Hsr => "hsr",
                Self::Unknown => "unknown",
                Self::Xfrm => "xfrm",
                Self::Other(s) => s,
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IfaceState {
    Up,
    Dormant,
    Down,
    LowerLayerDown,
    Testing,
    Absent, // Only for IfaceConf
    Other(String),
    #[default]
    Unknown,
}

impl From<link::State> for IfaceState {
    fn from(d: link::State) -> Self {
        match d {
            link::State::Up => Self::Up,
            link::State::Down => Self::Down,
            link::State::LowerLayerDown => Self::LowerLayerDown,
            link::State::Testing => Self::Testing,
            link::State::Dormant => Self::Dormant,
            link::State::Unknown => Self::Unknown,
            _ => {
                let mut s = format!("{d:?}");
                s.make_ascii_lowercase();
                Self::Other(s)
            }
        }
    }
}

impl std::fmt::Display for IfaceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Up => "up",
                Self::Dormant => "dormant",
                Self::Down => "down",
                Self::LowerLayerDown => "lower_layer_down",
                Self::Testing => "testing",
                Self::Absent => "absent",
                Self::Other(s) => s.as_str(),
                Self::Unknown => "unknown",
            }
        )
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum IfaceFlag {
    AllMulti,
    AutoMedia,
    Broadcast,
    Debug,
    Dormant,
    Loopback,
    LowerUp,
    Controller,
    Multicast,
    NoArp,
    PoinToPoint,
    Portsel,
    Promisc,
    Running,
    Subordinate,
    Up,
    Other(u32),
    #[default]
    Unknown,
}

impl From<link::LinkFlag> for IfaceFlag {
    fn from(d: link::LinkFlag) -> IfaceFlag {
        match d {
            link::LinkFlag::Allmulti => Self::AllMulti,
            link::LinkFlag::Automedia => Self::AutoMedia,
            link::LinkFlag::Broadcast => Self::Broadcast,
            link::LinkFlag::Debug => Self::Debug,
            link::LinkFlag::Dormant => Self::Dormant,
            link::LinkFlag::Loopback => Self::Loopback,
            link::LinkFlag::LowerUp => Self::LowerUp,
            link::LinkFlag::Controller => Self::Controller,
            link::LinkFlag::Multicast => Self::Multicast,
            link::LinkFlag::Noarp => Self::NoArp,
            link::LinkFlag::Pointopoint => Self::PoinToPoint,
            link::LinkFlag::Portsel => Self::Portsel,
            link::LinkFlag::Promisc => Self::Promisc,
            link::LinkFlag::Running => Self::Running,
            link::LinkFlag::Port => Self::Subordinate,
            link::LinkFlag::Up => Self::Up,
            _ => Self::Other(d.into()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ControllerType {
    Bond,
    Bridge,
    Vrf,
    OpenvSwitch,
    Other(String),
    Unknown,
}

impl From<&str> for ControllerType {
    fn from(s: &str) -> Self {
        match s {
            "bond" => ControllerType::Bond,
            "bridge" => ControllerType::Bridge,
            "vrf" => ControllerType::Vrf,
            "openvswitch" => ControllerType::OpenvSwitch,
            _ => ControllerType::Other(s.to_string()),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct Iface {
    pub name: String,
    #[serde(skip_serializing)]
    pub index: u32,
    pub iface_type: IfaceType,
    pub state: IfaceState,
    pub mtu: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mtu: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mtu: Option<i64>,
    pub flags: Vec<IfaceFlag>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<Ipv4Info>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<Ipv6Info>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mac_address: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub permanent_mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_type: Option<ControllerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_netnsid: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethtool: Option<EthtoolInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond: Option<BondInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_subordinate: Option<BondSubordinateInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<BridgeInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_vlan: Option<Vec<BridgeVlanEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_port: Option<BridgePortInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tun: Option<TunInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan: Option<VlanInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vxlan: Option<VxlanInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub veth: Option<VethInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf: Option<VrfInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vrf_subordinate: Option<VrfSubordinateInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_vlan: Option<MacVlanInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mac_vtap: Option<MacVtapInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sriov: Option<SriovInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sriov_vf: Option<VfInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipoib: Option<IpoibInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mptcp: Option<Vec<MptcpAddress>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub macsec: Option<MacSecInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hsr: Option<HsrInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xfrm: Option<XfrmInfo>,
}

// TODO: impl From Iface to IfaceConf

pub(crate) fn parse_nl_msg_to_name_and_index(
    nl_msg: &LinkMessage,
) -> Option<(String, u32)> {
    let index = nl_msg.header.index;
    let name = _get_iface_name(nl_msg);
    if name.is_empty() {
        None
    } else {
        Some((name, index))
    }
}

pub(crate) fn parse_nl_msg_to_iface(
    nl_msg: &LinkMessage,
) -> Result<Option<Iface>, NisporError> {
    let name = _get_iface_name(nl_msg);
    if name.is_empty() {
        return Ok(None);
    }
    let link_layer_type = match nl_msg.header.link_layer_type {
        LinkLayerType::Ether => IfaceType::Ethernet,
        LinkLayerType::Loopback => IfaceType::Loopback,
        LinkLayerType::Infiniband => IfaceType::Infiniband,
        _ => IfaceType::Unknown,
    };
    let mut iface_state = Iface {
        name,
        iface_type: link_layer_type.clone(),
        ..Default::default()
    };
    iface_state.index = nl_msg.header.index;
    let mut link: Option<u32> = None;
    for nla in &nl_msg.attributes {
        if let LinkAttribute::Mtu(mtu) = nla {
            iface_state.mtu = *mtu as i64;
        } else if let LinkAttribute::MinMtu(mtu) = nla {
            iface_state.min_mtu =
                if *mtu != 0 { Some(*mtu as i64) } else { None };
        } else if let LinkAttribute::MaxMtu(mtu) = nla {
            iface_state.max_mtu =
                if *mtu != 0 { Some(*mtu as i64) } else { None };
        } else if let LinkAttribute::Address(mac) = nla {
            iface_state.mac_address = parse_as_mac(mac.len(), mac)?;
        } else if let LinkAttribute::PermAddress(mac) = nla {
            iface_state.permanent_mac_address = parse_as_mac(mac.len(), mac)?;
        } else if let LinkAttribute::OperState(state) = nla {
            iface_state.state = (*state).into();
        } else if let LinkAttribute::Controller(controller) = nla {
            iface_state.controller = Some(format!("{controller}"));
        } else if let LinkAttribute::Link(l) = nla {
            link = Some(*l);
        } else if let LinkAttribute::LinkInfo(infos) = nla {
            for info in infos {
                if let LinkInfo::Kind(t) = info {
                    let iface_type = match t {
                        InfoKind::Bond => IfaceType::Bond,
                        InfoKind::Veth => IfaceType::Veth,
                        InfoKind::Bridge => IfaceType::Bridge,
                        InfoKind::Vlan => IfaceType::Vlan,
                        InfoKind::Vxlan => IfaceType::Vxlan,
                        InfoKind::Dummy => IfaceType::Dummy,
                        InfoKind::Tun => IfaceType::Tun,
                        InfoKind::Vrf => IfaceType::Vrf,
                        InfoKind::MacVlan => IfaceType::MacVlan,
                        InfoKind::MacVtap => IfaceType::MacVtap,
                        InfoKind::Ipoib => IfaceType::Ipoib,
                        InfoKind::MacSec => IfaceType::MacSec,
                        InfoKind::Hsr => IfaceType::Hsr,
                        InfoKind::Xfrm => IfaceType::Xfrm,
                        InfoKind::Other(s) => match s.as_ref() {
                            "openvswitch" => IfaceType::OpenvSwitch,
                            _ => IfaceType::Other(s.clone()),
                        },
                        _ => IfaceType::Other(
                            format!("{t:?}").as_str().to_lowercase(),
                        ),
                    };
                    if let IfaceType::Other(_) = iface_type {
                        /* We did not find an explicit link type. Instead it's
                         * just "Other(_)". If we already determined a link type
                         * above (ethernet or infiniband), keep that one. */
                        if iface_state.iface_type == IfaceType::Unknown {
                            iface_state.iface_type = iface_type
                        }
                    } else {
                        /* We found a better link type based on the kind. Use it. */
                        iface_state.iface_type = iface_type
                    }
                }
            }
            for info in infos {
                if let LinkInfo::Data(d) = info {
                    match iface_state.iface_type {
                        IfaceType::Bond => iface_state.bond = get_bond_info(d)?,
                        IfaceType::Bridge => {
                            iface_state.bridge = get_bridge_info(d)?
                        }
                        IfaceType::Tun => match get_tun_info(d) {
                            Ok(info) => {
                                iface_state.tun = Some(info);
                            }
                            Err(e) => {
                                log::warn!("Error parsing TUN info: {}", e);
                            }
                        },
                        IfaceType::Vlan => iface_state.vlan = get_vlan_info(d),
                        IfaceType::Vxlan => {
                            iface_state.vxlan = get_vxlan_info(d)?
                        }
                        IfaceType::Vrf => iface_state.vrf = get_vrf_info(d),
                        IfaceType::MacVlan => {
                            iface_state.mac_vlan = get_mac_vlan_info(d)?
                        }
                        IfaceType::MacVtap => {
                            iface_state.mac_vtap = get_mac_vtap_info(d)?
                        }
                        IfaceType::Ipoib => {
                            iface_state.ipoib = get_ipoib_info(d);
                        }
                        IfaceType::MacSec => {
                            iface_state.macsec = get_macsec_info(d);
                        }
                        IfaceType::Hsr => {
                            iface_state.hsr = get_hsr_info(d);
                        }
                        IfaceType::Xfrm => {
                            iface_state.xfrm = get_xfrm_info(d);
                        }
                        _ => log::warn!(
                            "Unhandled IFLA_INFO_DATA for iface type {:?}",
                            iface_state.iface_type
                        ),
                    }
                }
            }
            for info in infos {
                if let LinkInfo::PortKind(d) = info {
                    match d {
                        InfoPortKind::Bond => {
                            iface_state.controller_type =
                                Some(ControllerType::Bond)
                        }
                        InfoPortKind::Other(s) => {
                            iface_state.controller_type =
                                Some(s.as_str().into())
                        }
                        _ => {
                            log::info!("Unknown port kind {:?}", info);
                        }
                    }
                }
            }
            if let Some(controller_type) = &iface_state.controller_type {
                for info in infos {
                    if let LinkInfo::PortData(d) = info {
                        match d {
                            InfoPortData::BondPort(bond_ports) => {
                                iface_state.bond_subordinate = Some(
                                    get_bond_subordinate_info(bond_ports)?,
                                );
                            }
                            InfoPortData::Other(data) => {
                                match controller_type {
                                    ControllerType::Bridge => {
                                        iface_state.bridge_port =
                                            get_bridge_port_info(data)?;
                                    }
                                    ControllerType::Vrf => {
                                        iface_state.vrf_subordinate =
                                            get_vrf_subordinate_info(data)?;
                                    }
                                    _ => log::warn!(
                                        "Unknown controller type {:?}",
                                        controller_type
                                    ),
                                }
                            }
                            _ => {
                                log::debug!("Unknown InfoPortData {:?}", d);
                            }
                        }
                    }
                }
            }
        } else if let LinkAttribute::VfInfoList(nlas) = nla {
            if let Ok(info) =
                get_sriov_info(&iface_state.name, nlas, &link_layer_type)
            {
                iface_state.sriov = Some(info);
            }
        } else if let LinkAttribute::NetnsId(id) = nla {
            iface_state.link_netnsid = Some(*id);
        } else if let LinkAttribute::AfSpecUnspec(nlas) = nla {
            fill_af_spec_inet_info(&mut iface_state, nlas);
        } else {
            // Place holder for paring more Nla
        }
    }
    if let Some(ref mut vlan_info) = iface_state.vlan {
        if let Some(base_iface_index) = link {
            vlan_info.base_iface = format!("{base_iface_index}");
        }
    }
    if let Some(ref mut ib_info) = iface_state.ipoib {
        if let Some(base_iface_index) = link {
            ib_info.base_iface = Some(format!("{base_iface_index}"));
        }
    }
    if let Some(ref mut macsec_info) = iface_state.macsec {
        if let Some(base_iface_index) = link {
            macsec_info.base_iface = Some(format!("{base_iface_index}"));
        }
    }
    if let Some(iface_index) = link {
        match iface_state.iface_type {
            IfaceType::Veth => {
                iface_state.veth = Some(VethInfo {
                    peer: format!("{iface_index}"),
                })
            }
            IfaceType::MacVlan => {
                if let Some(ref mut mac_vlan_info) = iface_state.mac_vlan {
                    mac_vlan_info.base_iface = format!("{iface_index}");
                }
            }
            IfaceType::MacVtap => {
                if let Some(ref mut mac_vtap_info) = iface_state.mac_vtap {
                    mac_vtap_info.base_iface = format!("{iface_index}");
                }
            }
            _ => (),
        }
    }
    iface_state.flags = nl_msg
        .header
        .flags
        .as_slice()
        .iter()
        .map(|f| IfaceFlag::from(*f))
        .collect();
    Ok(Some(iface_state))
}

fn _get_iface_name(nl_msg: &LinkMessage) -> String {
    for nla in &nl_msg.attributes {
        if let LinkAttribute::IfName(name) = nla {
            return name.clone();
        }
    }
    "".into()
}

pub(crate) fn fill_bridge_vlan_info(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &LinkMessage,
) -> Result<(), NisporError> {
    let name = _get_iface_name(nl_msg);
    if name.is_empty() {
        return Ok(());
    }
    if let Some(iface_state) = iface_states.get_mut(&name) {
        for nla in &nl_msg.attributes {
            if let LinkAttribute::AfSpecBridge(nlas) = nla {
                parse_bridge_vlan_info(iface_state, nlas)?;
            }
        }
    }
    Ok(())
}
