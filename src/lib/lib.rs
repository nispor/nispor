// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod crate_tests;
mod error;
mod filter;
mod iface_filter;
// Since rust 1.62, the `#[default]` can be used for setting default value of
// `#[derive(Default)]` for enum. The cargo clippy will complain if we impl the
// Default by ourselves. But currently nispor minimum rust version is 1.58,
// hence we suppress the clippy warning here.
#[allow(clippy::derivable_impls)]
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
mod route_rule_filter;

pub use crate::error::NisporError;
pub use crate::filter::NetStateFilter;
pub use crate::iface_filter::NetStateIfaceFilter;
pub use crate::ifaces::{
    BondAdInfo, BondAdSelect, BondAllSubordinatesActive, BondArpValidate,
    BondConf, BondFailOverMac, BondInfo, BondLacpRate, BondMiiStatus, BondMode,
    BondModeArpAllTargets, BondPrimaryReselect, BondSubordinateInfo,
    BondSubordinateState, BondXmitHashPolicy, BridgeConf, BridgeInfo,
    BridgePortInfo, BridgePortMulticastRouterType, BridgePortStpState,
    BridgeStpState, BridgeVlanEntry, BridgeVlanProtocol, ControllerType,
    EthtoolCoalesceInfo, EthtoolFeatureInfo, EthtoolInfo,
    EthtoolLinkModeDuplex, EthtoolLinkModeInfo, EthtoolPauseInfo,
    EthtoolRingInfo, Iface, IfaceConf, IfaceFlags, IfaceState, IfaceType,
    IpoibInfo, IpoibMode, MacVlanInfo, MacVlanMode, MacVtapInfo, MacVtapMode,
    SriovInfo, TunInfo, TunMode, VethConf, VethInfo, VfInfo, VfLinkState,
    VfState, VlanConf, VlanInfo, VlanProtocol, VrfInfo, VrfSubordinateInfo,
    VxlanInfo,
};
pub use crate::ip::{
    IpAddrConf, IpConf, IpFamily, Ipv4AddrInfo, Ipv4Info, Ipv6AddrInfo,
    Ipv6Info,
};
pub use crate::mptcp::{Mptcp, MptcpAddress, MptcpAddressFlag};
pub use crate::net_conf::NetConf;
pub use crate::net_state::NetState;
pub use crate::route::{
    AddressFamily, MultipathRoute, MultipathRouteFlags, Route, RouteConf,
    RouteProtocol, RouteScope, RouteType,
};
pub use crate::route_filter::NetStateRouteFilter;
pub use crate::route_rule::{RouteRule, RuleAction};
pub use crate::route_rule_filter::NetStateRouteRuleFilter;
