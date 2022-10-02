// SPDX-License-Identifier: Apache-2.0

mod error;
mod ifaces;
mod ip;
mod mac;
mod mptcp;
mod net_conf;
mod net_state;
mod netlink;
mod route;
mod route_filter;
mod route_rule;

pub use crate::error::NisporError;
pub use crate::ifaces::{
    BondAdInfo, BondAdSelect, BondAllSubordinatesActive, BondArpValidate,
    BondFailOverMac, BondInfo, BondLacpRate, BondMiiStatus, BondMode,
    BondModeArpAllTargets, BondPrimaryReselect, BondSubordinateInfo,
    BondSubordinateState, BondXmitHashPolicy, BridgeInfo, BridgePortInfo,
    BridgePortMulticastRouterType, BridgePortStpState, BridgeStpState,
    BridgeVlanEntry, BridgeVlanProtocol, ControllerType, EthtoolCoalesceInfo,
    EthtoolFeatureInfo, EthtoolInfo, EthtoolLinkModeDuplex,
    EthtoolLinkModeInfo, EthtoolPauseInfo, EthtoolRingInfo, Iface, IfaceConf,
    IfaceFlags, IfaceState, IfaceType, IpoibInfo, IpoibMode, MacVlanInfo,
    MacVlanMode, MacVtapInfo, MacVtapMode, SriovInfo, TunInfo, TunMode,
    VethConf, VethInfo, VfInfo, VfLinkState, VfState, VlanConf, VlanInfo,
    VlanProtocol, VrfInfo, VrfSubordinateInfo, VxlanInfo,
};
pub use crate::ip::{
    IpAddrConf, IpConf, IpFamily, Ipv4AddrInfo, Ipv4Info, Ipv6AddrInfo,
    Ipv6Info,
};
pub use crate::mptcp::{Mptcp, MptcpAddress, MptcpAddressFlag};
pub use crate::net_conf::NetConf;
pub use crate::net_state::{retrieve_routes_with_filter, NetState};
pub use crate::route::{
    AddressFamily, MultipathRoute, MultipathRouteFlags, Route, RouteConf,
    RouteProtocol, RouteScope, RouteType,
};
pub use crate::route_filter::NetStateRouteFilter;
pub use crate::route_rule::{RouteRule, RuleAction};
