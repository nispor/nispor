use crate::ifaces::bond::get_bond_info;
use crate::ifaces::bond::get_bond_subordinate_info;
use crate::ifaces::bond::BondInfo;
use crate::ifaces::bond::BondSubordinateInfo;
use crate::ifaces::bridge::get_bridge_info;
use crate::ifaces::bridge::get_bridge_port_info;
use crate::ifaces::bridge::parse_bridge_vlan_info;
use crate::ifaces::bridge::BridgeInfo;
use crate::ifaces::bridge::BridgePortInfo;
use crate::ifaces::ethtool::EthtoolInfo;
use crate::ifaces::mac_vlan::get_mac_vlan_info;
use crate::ifaces::mac_vlan::MacVlanInfo;
use crate::ifaces::mac_vtap::get_mac_vtap_info;
use crate::ifaces::mac_vtap::MacVtapInfo;
use crate::ifaces::sriov::get_sriov_info;
use crate::ifaces::sriov::SriovInfo;
use crate::ifaces::tun::get_tun_info;
use crate::ifaces::tun::TunInfo;
use crate::ifaces::veth::VethInfo;
use crate::ifaces::vlan::get_vlan_info;
use crate::ifaces::vlan::VlanInfo;
use crate::ifaces::vrf::get_vrf_info;
use crate::ifaces::vrf::get_vrf_subordinate_info;
use crate::ifaces::vrf::VrfInfo;
use crate::ifaces::vrf::VrfSubordinateInfo;
use crate::ifaces::vxlan::get_vxlan_info;
use crate::ifaces::vxlan::VxlanInfo;
use crate::mac::parse_as_mac;
use crate::IpConf;
use crate::IpFamily;
use crate::Ipv4Info;
use crate::Ipv6Info;
use crate::NisporError;
use netlink_packet_route::rtnl::link::nlas;
use netlink_packet_route::rtnl::LinkMessage;
use netlink_packet_route::rtnl::{
    ARPHRD_ETHER, IFF_ALLMULTI, IFF_AUTOMEDIA, IFF_BROADCAST, IFF_DEBUG,
    IFF_DORMANT, IFF_LOOPBACK, IFF_LOWER_UP, IFF_MASTER, IFF_MULTICAST,
    IFF_NOARP, IFF_POINTOPOINT, IFF_PORTSEL, IFF_PROMISC, IFF_RUNNING,
    IFF_SLAVE, IFF_UP,
};
use rtnetlink::new_connection;

use rtnetlink::packet::rtnl::link::nlas::Nla;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IfaceType {
    Bond,
    Veth,
    Bridge,
    Vlan,
    Dummy,
    Vxlan,
    Loopback,
    Ethernet,
    Vrf,
    Tun,
    MacVlan,
    MacVtap,
    OpenvSwitch,
    Unknown,
    Other(String),
}

impl Default for IfaceType {
    fn default() -> Self {
        IfaceType::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IfaceState {
    Up,
    Dormant,
    Down,
    LowerLayerDown,
    Other(String),
    Unknown,
}

impl Default for IfaceState {
    fn default() -> Self {
        IfaceState::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum IfaceFlags {
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
    Unknown,
}

impl Default for IfaceFlags {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct Iface {
    pub name: String,
    #[serde(skip_serializing)]
    pub index: u32,
    pub iface_type: IfaceType,
    pub state: IfaceState,
    pub mtu: i64,
    pub flags: Vec<IfaceFlags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv4: Option<Ipv4Info>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ipv6: Option<Ipv6Info>,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mac_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controller_type: Option<ControllerType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethtool: Option<EthtoolInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond: Option<BondInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bond_subordinate: Option<BondSubordinateInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<BridgeInfo>,
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
}

// TODO: impl From Iface to IfaceConf

pub(crate) fn get_iface_name_by_index(
    iface_states: &HashMap<String, Iface>,
    iface_index: u32,
) -> String {
    for (iface_name, iface) in iface_states.iter() {
        if iface.index == iface_index {
            return iface_name.clone();
        }
    }
    "".into()
}

pub(crate) fn parse_nl_msg_to_iface(
    nl_msg: &LinkMessage,
) -> Result<Option<Iface>, NisporError> {
    let name = _get_iface_name(&nl_msg);
    if name.len() <= 0 {
        return Ok(None);
    }
    let mut iface_state = Iface {
        name: name.clone(),
        ..Default::default()
    };
    if nl_msg.header.link_layer_type == ARPHRD_ETHER {
        iface_state.iface_type = IfaceType::Ethernet
    }
    iface_state.index = nl_msg.header.index;
    let mut link: Option<u32> = None;
    let mut mac_len = None;
    for nla in &nl_msg.nlas {
        if let Nla::Mtu(mtu) = nla {
            iface_state.mtu = *mtu as i64;
        } else if let Nla::Address(mac) = nla {
            mac_len = Some(mac.len());
            iface_state.mac_address = parse_as_mac(mac.len(), mac)?;
        } else if let Nla::OperState(state) = nla {
            iface_state.state = _get_iface_state(&state);
        } else if let Nla::Master(controller) = nla {
            iface_state.controller = Some(format!("{}", controller));
        } else if let Nla::Link(l) = nla {
            link = Some(*l);
        } else if let Nla::Info(infos) = nla {
            for info in infos {
                if let nlas::Info::Kind(t) = info {
                    iface_state.iface_type = match t {
                        nlas::InfoKind::Bond => IfaceType::Bond,
                        nlas::InfoKind::Veth => IfaceType::Veth,
                        nlas::InfoKind::Bridge => IfaceType::Bridge,
                        nlas::InfoKind::Vlan => IfaceType::Vlan,
                        nlas::InfoKind::Vxlan => IfaceType::Vxlan,
                        nlas::InfoKind::Dummy => IfaceType::Dummy,
                        nlas::InfoKind::Tun => IfaceType::Tun,
                        nlas::InfoKind::Vrf => IfaceType::Vrf,
                        nlas::InfoKind::MacVlan => IfaceType::MacVlan,
                        nlas::InfoKind::MacVtap => IfaceType::MacVtap,
                        nlas::InfoKind::Other(s) => match s.as_ref() {
                            "openvswitch" => IfaceType::OpenvSwitch,
                            _ => IfaceType::Other(s.clone()),
                        },
                        _ => IfaceType::Other(format!("{:?}", t)),
                    };
                }
            }
            for info in infos {
                if let nlas::Info::Data(d) = info {
                    match iface_state.iface_type {
                        IfaceType::Bond => {
                            iface_state.bond = get_bond_info(&d)?
                        }
                        IfaceType::Bridge => {
                            iface_state.bridge = get_bridge_info(&d)?
                        }
                        IfaceType::Tun => match get_tun_info(&d) {
                            Ok(info) => {
                                iface_state.tun = Some(info);
                            }
                            Err(e) => {
                                eprintln!("Error parsing TUN info: {}", e);
                            }
                        },
                        IfaceType::Vlan => iface_state.vlan = get_vlan_info(&d),
                        IfaceType::Vxlan => {
                            iface_state.vxlan = get_vxlan_info(&d)?
                        }
                        IfaceType::Vrf => iface_state.vrf = get_vrf_info(&d),
                        IfaceType::MacVlan => {
                            iface_state.mac_vlan = get_mac_vlan_info(&d)?
                        }
                        IfaceType::MacVtap => {
                            iface_state.mac_vtap = get_mac_vtap_info(&d)?
                        }
                        _ => eprintln!(
                            "Unhandled IFLA_INFO_DATA for iface type {:?}",
                            iface_state.iface_type
                        ),
                    }
                }
            }
            for info in infos {
                if let nlas::Info::SlaveKind(d) = info {
                    // Remove the tailing \0
                    iface_state.controller_type = Some(
                        std::ffi::CStr::from_bytes_with_nul(&d.as_slice())?
                            .to_str()?
                            .into(),
                    )
                }
            }
            if let Some(controller_type) = &iface_state.controller_type {
                for info in infos {
                    if let nlas::Info::SlaveData(d) = info {
                        match controller_type {
                            ControllerType::Bond => {
                                iface_state.bond_subordinate =
                                    get_bond_subordinate_info(&d)?;
                            }
                            ControllerType::Bridge => {
                                iface_state.bridge_port =
                                    get_bridge_port_info(&d)?;
                            }
                            ControllerType::Vrf => {
                                iface_state.vrf_subordinate =
                                    get_vrf_subordinate_info(&d)?;
                            }
                            _ => eprintln!(
                                "Unknown controller type {:?}",
                                controller_type
                            ),
                        }
                    }
                }
            }
        } else if let Nla::VfInfoList(data) = nla {
            if let Ok(info) = get_sriov_info(data, mac_len) {
                iface_state.sriov = Some(info);
            }
        } else {
            // println!("{} {:?}", name, nla);
        }
    }
    if let Some(ref mut vlan_info) = iface_state.vlan {
        if let Some(base_iface_index) = link {
            vlan_info.base_iface = format!("{}", base_iface_index);
        }
    }
    if let Some(iface_index) = link {
        match iface_state.iface_type {
            IfaceType::Veth => {
                iface_state.veth = Some(VethInfo {
                    peer: format!("{}", iface_index),
                })
            }
            IfaceType::MacVlan => {
                if let Some(ref mut mac_vlan_info) = iface_state.mac_vlan {
                    mac_vlan_info.base_iface = format!("{}", iface_index);
                }
            }
            IfaceType::MacVtap => {
                if let Some(ref mut mac_vtap_info) = iface_state.mac_vtap {
                    mac_vtap_info.base_iface = format!("{}", iface_index);
                }
            }
            _ => (),
        }
    }
    if (nl_msg.header.flags & IFF_LOOPBACK) > 0 {
        iface_state.iface_type = IfaceType::Loopback;
    }
    iface_state.flags = _parse_iface_flags(nl_msg.header.flags);
    Ok(Some(iface_state))
}

fn _get_iface_name(nl_msg: &LinkMessage) -> String {
    for nla in &nl_msg.nlas {
        if let Nla::IfName(name) = nla {
            return name.clone();
        }
    }
    "".into()
}

pub(crate) fn fill_bridge_vlan_info(
    iface_states: &mut HashMap<String, Iface>,
    nl_msg: &LinkMessage,
) -> Result<(), NisporError> {
    let name = _get_iface_name(&nl_msg);
    if name.len() <= 0 {
        return Ok(());
    }
    if let Some(mut iface_state) = iface_states.get_mut(&name) {
        for nla in &nl_msg.nlas {
            if let Nla::AfSpecBridge(data) = nla {
                parse_bridge_vlan_info(&mut iface_state, &data)?;
                break;
            }
        }
    }
    Ok(())
}

fn _get_iface_state(state: &nlas::State) -> IfaceState {
    match state {
        nlas::State::Up => IfaceState::Up,
        nlas::State::Dormant => IfaceState::Dormant,
        nlas::State::Down => IfaceState::Down,
        nlas::State::LowerLayerDown => IfaceState::LowerLayerDown,
        nlas::State::Unknown => IfaceState::Unknown,
        _ => IfaceState::Other(format!("{:?}", state)),
    }
}

fn _parse_iface_flags(flags: u32) -> Vec<IfaceFlags> {
    let mut ret = Vec::new();
    if (flags & IFF_ALLMULTI) > 0 {
        ret.push(IfaceFlags::AllMulti)
    }
    if (flags & IFF_AUTOMEDIA) > 0 {
        ret.push(IfaceFlags::AutoMedia)
    }
    if (flags & IFF_BROADCAST) > 0 {
        ret.push(IfaceFlags::Broadcast)
    }
    if (flags & IFF_DEBUG) > 0 {
        ret.push(IfaceFlags::Debug)
    }
    if (flags & IFF_DORMANT) > 0 {
        ret.push(IfaceFlags::Dormant)
    }
    if (flags & IFF_LOOPBACK) > 0 {
        ret.push(IfaceFlags::Loopback)
    }
    if (flags & IFF_LOWER_UP) > 0 {
        ret.push(IfaceFlags::LowerUp)
    }
    if (flags & IFF_MASTER) > 0 {
        ret.push(IfaceFlags::Controller)
    }
    if (flags & IFF_MULTICAST) > 0 {
        ret.push(IfaceFlags::Multicast)
    }
    if (flags & IFF_NOARP) > 0 {
        ret.push(IfaceFlags::NoArp)
    }
    if (flags & IFF_POINTOPOINT) > 0 {
        ret.push(IfaceFlags::PoinToPoint)
    }
    if (flags & IFF_PORTSEL) > 0 {
        ret.push(IfaceFlags::Portsel)
    }
    if (flags & IFF_PROMISC) > 0 {
        ret.push(IfaceFlags::Promisc)
    }
    if (flags & IFF_RUNNING) > 0 {
        ret.push(IfaceFlags::Running)
    }
    if (flags & IFF_SLAVE) > 0 {
        ret.push(IfaceFlags::Subordinate)
    }
    if (flags & IFF_UP) > 0 {
        ret.push(IfaceFlags::Up)
    }

    ret
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct IfaceConf {
    pub name: String,
    pub iface_type: Option<IfaceType>,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
}

impl IfaceConf {
    // pub async fn create() { }
    pub async fn apply(&self, cur_iface: &Iface) -> Result<(), NisporError> {
        let (connection, handle, _) = new_connection()?;
        tokio::spawn(connection);
        if let Some(ipv6_conf) = &self.ipv6 {
            ipv6_conf.apply(&handle, &cur_iface, IpFamily::Ipv6).await?;
        } else {
            IpConf {
                addresses: Vec::new(),
            }
            .apply(&handle, &cur_iface, IpFamily::Ipv6)
            .await?;
        }
        if let Some(ipv4_conf) = &self.ipv4 {
            ipv4_conf.apply(&handle, &cur_iface, IpFamily::Ipv4).await?;
        } else {
            IpConf {
                addresses: Vec::new(),
            }
            .apply(&handle, &cur_iface, IpFamily::Ipv4)
            .await?;
        }
        Ok(())
    }
}
