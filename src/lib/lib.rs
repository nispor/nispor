// SPDX-License-Identifier: Apache-2.0

mod conf;
mod error;
mod filter;
#[cfg(test)]
mod integ_tests;
mod mac;
mod net_conf;
mod net_state;
mod netlink;
mod query;

pub use crate::{
    conf::{
        AltNameConf, BondConf, BridgeConf, IfaceConf, IpAddrConf, IpConf,
        RouteConf, VethConf, VlanConf,
    },
    error::{ErrorKind, NisporError},
    filter::{
        NetStateFilter, NetStateIfaceFilter, NetStateRouteFilter,
        NetStateRouteRuleFilter,
    },
    net_conf::NetConf,
    net_state::NetState,
    query::{
        AddressFamily, BondAdInfo, BondAdSelect, BondAllSubordinatesActive,
        BondArpValidate, BondFailOverMac, BondInfo, BondLacpRate,
        BondMiiStatus, BondMode, BondModeArpAllTargets, BondPrimaryReselect,
        BondSubordinateInfo, BondSubordinateState, BondXmitHashPolicy,
        BridgeInfo, BridgePortInfo, BridgePortMulticastRouterType,
        BridgePortStpState, BridgeStpState, BridgeVlanEntry,
        BridgeVlanProtocol, ControllerType, EthtoolCoalesceInfo,
        EthtoolFeatureInfo, EthtoolFecInfo, EthtoolFecMode, EthtoolInfo,
        EthtoolLinkModeDuplex, EthtoolLinkModeInfo, EthtoolPauseInfo,
        EthtoolRingInfo, HsrInfo, HsrProtocol, Iface, IfaceFlag, IfaceState,
        IfaceType, IpFamily, IpVlanFlag, IpVlanInfo, IpVlanMode, IpoibInfo,
        IpoibMode, Ipv4AddrInfo, Ipv4Info, Ipv6AddrFlag, Ipv6AddrInfo,
        Ipv6Info, MacSecCipherId, MacSecInfo, MacSecOffload, MacSecValidate,
        MacVlanInfo, MacVlanMode, MacVtapInfo, MacVtapMode, Mptcp,
        MptcpAddress, MptcpAddressFlag, MultipathRoute, MultipathRouteFlags,
        PciAddress, Route, RouteProtocol, RouteRule, RouteScope, RouteType,
        RuleAction, SriovInfo, TunInfo, TunMode, VethInfo, VfInfo, VfLinkState,
        VfState, VlanInfo, VlanProtocol, VlanQosMapping, VrfInfo,
        VrfSubordinateInfo, VxlanInfo, WifiInfo, XfrmInfo,
    },
};
