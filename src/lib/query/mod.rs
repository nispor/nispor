// SPDX-License-Identifier: Apache-2.0

mod bond;
mod bridge;
mod ip;
mod mptcp;
// Disable `needless_pass_by_ref_mut` check due to upstream issue:
// https://github.com/rust-netlink/ethtool/issues/12
#[allow(clippy::needless_pass_by_ref_mut)]
mod ethtool;
mod iface;
mod inter_ifaces;
mod ipoib;
mod mac_vlan;
mod mac_vtap;
mod macsec;
mod route;
mod route_rule;
mod sriov;
mod tun;
mod veth;
mod vlan;
mod vrf;
mod vxlan;

pub use self::bond::{
    BondAdInfo, BondAdSelect, BondAllSubordinatesActive, BondArpValidate,
    BondFailOverMac, BondInfo, BondLacpRate, BondMiiStatus, BondMode,
    BondModeArpAllTargets, BondPrimaryReselect, BondSubordinateInfo,
    BondSubordinateState, BondXmitHashPolicy,
};
pub use self::bridge::{
    BridgeInfo, BridgePortInfo, BridgePortMulticastRouterType,
    BridgePortStpState, BridgeStpState, BridgeVlanEntry, BridgeVlanProtocol,
};
pub use self::ethtool::{
    EthtoolCoalesceInfo, EthtoolFeatureInfo, EthtoolInfo,
    EthtoolLinkModeDuplex, EthtoolLinkModeInfo, EthtoolPauseInfo,
    EthtoolRingInfo,
};
pub use self::iface::{
    ControllerType, Iface, IfaceFlags, IfaceState, IfaceType,
};
pub use self::ip::{IpFamily, Ipv4AddrInfo, Ipv4Info, Ipv6AddrInfo, Ipv6Info};
pub use self::ipoib::{IpoibInfo, IpoibMode};
pub use self::mac_vlan::{MacVlanInfo, MacVlanMode};
pub use self::mac_vtap::{MacVtapInfo, MacVtapMode};
pub use self::macsec::{
    MacSecCipherId, MacSecInfo, MacSecOffload, MacSecValidate,
};
pub use self::mptcp::{Mptcp, MptcpAddress, MptcpAddressFlag};
pub use self::route::{
    AddressFamily, MultipathRoute, MultipathRouteFlags, Route, RouteProtocol,
    RouteScope, RouteType,
};
pub use self::route_rule::{RouteRule, RuleAction};
pub use self::sriov::{SriovInfo, VfInfo, VfLinkState, VfState};
pub use self::tun::{TunInfo, TunMode};
pub use self::veth::VethInfo;
pub use self::vlan::{VlanInfo, VlanProtocol};
pub use self::vrf::{VrfInfo, VrfSubordinateInfo};
pub use self::vxlan::VxlanInfo;

pub(crate) use self::{
    inter_ifaces::{get_iface_name2index, get_ifaces},
    ip::{is_ipv6_addr, parse_ip_addr_str, parse_ip_net_addr_str},
    mptcp::{get_mptcp, merge_mptcp_info},
    route::get_routes,
    route_rule::get_route_rules,
};
