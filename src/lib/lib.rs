// SPDX-License-Identifier: Apache-2.0

mod error;
#[cfg(test)]
mod integ_tests;
mod mac;
// Since rust 1.62, the `#[default]` can be used for setting default value of
// `#[derive(Default)]` for enum. The cargo clippy will complain if we impl the
// Default by ourselves. But currently nispor minimum rust version is 1.58,
// hence we suppress the clippy warning here.
mod conf;
mod filter;
mod net_conf;
mod net_state;
mod netlink;
#[allow(clippy::derivable_impls)]
mod query;

pub use crate::conf::{
    BondConf, BridgeConf, IfaceConf, IpAddrConf, IpConf, RouteConf, VethConf,
    VlanConf,
};
pub use crate::error::{ErrorKind, NisporError};
pub use crate::filter::{
    NetStateFilter, NetStateIfaceFilter, NetStateRouteFilter,
    NetStateRouteRuleFilter,
};
pub use crate::net_conf::NetConf;
pub use crate::net_state::NetState;
pub use crate::query::{
    AddressFamily, BondAdInfo, BondAdSelect, BondAllSubordinatesActive,
    BondArpValidate, BondFailOverMac, BondInfo, BondLacpRate, BondMiiStatus,
    BondMode, BondModeArpAllTargets, BondPrimaryReselect, BondSubordinateInfo,
    BondSubordinateState, BondXmitHashPolicy, BridgeInfo, BridgePortInfo,
    BridgePortMulticastRouterType, BridgePortStpState, BridgeStpState,
    BridgeVlanEntry, BridgeVlanProtocol, ControllerType, EthtoolCoalesceInfo,
    EthtoolFeatureInfo, EthtoolInfo, EthtoolLinkModeDuplex,
    EthtoolLinkModeInfo, EthtoolPauseInfo, EthtoolRingInfo, Iface, IfaceFlag,
    IfaceState, IfaceType, IpFamily, IpoibInfo, IpoibMode, Ipv4AddrInfo,
    Ipv4Info, Ipv6AddrFlag, Ipv6AddrInfo, Ipv6Info, MacSecCipherId, MacSecInfo,
    MacSecOffload, MacSecValidate, MacVlanInfo, MacVlanMode, MacVtapInfo,
    MacVtapMode, Mptcp, MptcpAddress, MptcpAddressFlag, MultipathRoute,
    MultipathRouteFlags, Route, RouteProtocol, RouteRule, RouteScope,
    RouteType, RuleAction, SriovInfo, TunInfo, TunMode, VethInfo, VfInfo,
    VfLinkState, VfState, VlanInfo, VlanProtocol, VrfInfo, VrfSubordinateInfo,
    VxlanInfo,
};
