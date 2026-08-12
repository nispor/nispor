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
        AltNameConf, BondConf, BondPortConf, BridgeConf, BridgePortConf,
        DummyConf, IfaceConf, IpAddrConf, IpConf, RouteConf, VethConf,
        VlanConf, VrfConf, VxlanConf, WireguardConf, WireguardPeerConf,
    },
    error::{ErrorKind, NisporError},
    filter::{
        NetStateFilter, NetStateIfaceFilter, NetStateRouteFilter,
        NetStateRouteRuleFilter,
    },
    net_conf::NetConf,
    net_state::NetState,
    query::{
        AddressFamily, AddressProtocol, AddressScope, BondAdInfo, BondAdSelect,
        BondAllPortActive, BondArpValidate, BondFailOverMac, BondInfo,
        BondLacpRate, BondMiiStatus, BondMode, BondModeArpAllTargets,
        BondPortInfo, BondPortState, BondPrimaryReselect, BondXmitHashPolicy,
        BridgeInfo, BridgeMulticastRouterType, BridgePortInfo,
        BridgePortStpState, BridgeStpState, BridgeVlanEntry, ControllerType,
        EthtoolCoalesceInfo, EthtoolFeatureInfo, EthtoolFecInfo,
        EthtoolFecMode, EthtoolInfo, EthtoolLinkModeDuplex,
        EthtoolLinkModeInfo, EthtoolPauseInfo, EthtoolRingInfo, HsrInfo,
        HsrProtocol, Iface, IfaceFlag, IfaceState, IfaceType, Ip6TunnelFlag,
        IpAddrFlag, IpFamily, IpTunnelInfo, IpTunnelMode, IpVlanFlag,
        IpVlanInfo, IpVlanMode, IpoibInfo, IpoibMode, Ipv4AddrInfo, Ipv4Info,
        Ipv6AddrInfo, Ipv6Info, MacSecCipherId, MacSecInfo, MacSecOffload,
        MacSecValidate, MacVlanFlag, MacVlanInfo, MacVlanMode, MacVtapFlag,
        MacVtapInfo, MacVtapMode, Mptcp, MptcpAddress, MptcpAddressFlag,
        MultipathRoute, MultipathRouteFlag, PciAddress, Route, RouteFlag,
        RouteProtocol, RouteRealm, RouteRule, RouteRuleFlag, RouteScope,
        RouteType, RuleAction, SriovInfo, TunInfo, TunMode, TunnelEncapFlag,
        TunnelEncapType, VethInfo, VfInfo, VfLinkState, VfState, VlanInfo,
        VlanProtocol, VlanQosMapping, VrfInfo, VrfPortInfo, VxlanInfo,
        WifiInfo, WifiMode, WireguardInfo, WireguardIpAddress,
        WireguardPeerInfo, XfrmInfo,
    },
};
