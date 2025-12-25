// SPDX-License-Identifier: Apache-2.0

mod bond;
mod bridge;
mod bridge_vlan;
mod hsr;
mod ip;
mod iptunnel;
mod mptcp;
mod pci;
mod wifi;
mod xfrm;
// Disable `needless_pass_by_ref_mut` check due to upstream issue:
// https://github.com/rust-netlink/ethtool/issues/12
#[allow(clippy::needless_pass_by_ref_mut)]
mod ethtool;
mod iface;
mod inter_ifaces;
mod ipoib;
mod ipvlan;
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

pub use self::{
    bond::{
        BondAdInfo, BondAdSelect, BondAllPortsActive, BondArpValidate,
        BondFailOverMac, BondInfo, BondLacpRate, BondMiiStatus, BondMode,
        BondModeArpAllTargets, BondPortInfo, BondPortState,
        BondPrimaryReselect, BondXmitHashPolicy,
    },
    bridge::{
        BridgeInfo, BridgeMulticastRouterType, BridgePortInfo,
        BridgePortStpState, BridgeStpState,
    },
    bridge_vlan::BridgeVlanEntry,
    ethtool::{
        EthtoolCoalesceInfo, EthtoolFeatureInfo, EthtoolFecInfo,
        EthtoolFecMode, EthtoolInfo, EthtoolLinkModeDuplex,
        EthtoolLinkModeInfo, EthtoolPauseInfo, EthtoolRingInfo,
    },
    hsr::{HsrInfo, HsrProtocol},
    iface::{ControllerType, Iface, IfaceFlag, IfaceState, IfaceType},
    ip::{
        IpFamily, Ipv4AddrInfo, Ipv4Info, Ipv6AddrFlag, Ipv6AddrInfo, Ipv6Info,
    },
    ipoib::{IpoibInfo, IpoibMode},
    iptunnel::{
        Ip6TunnelFlags, IpTunnelInfo, IpTunnelMode, TunnelEncapFlags,
        TunnelEncapType,
    },
    ipvlan::{IpVlanFlag, IpVlanInfo, IpVlanMode},
    mac_vlan::{MacVlanInfo, MacVlanMode},
    mac_vtap::{MacVtapInfo, MacVtapMode},
    macsec::{MacSecCipherId, MacSecInfo, MacSecOffload, MacSecValidate},
    mptcp::{Mptcp, MptcpAddress, MptcpAddressFlag},
    pci::PciAddress,
    route::{
        AddressFamily, MultipathRoute, MultipathRouteFlags, Route,
        RouteProtocol, RouteScope, RouteType,
    },
    route_rule::{RouteRule, RuleAction},
    sriov::{SriovInfo, VfInfo, VfLinkState, VfState},
    tun::{TunInfo, TunMode},
    veth::VethInfo,
    vlan::{VlanInfo, VlanProtocol, VlanQosMapping},
    vrf::{VrfInfo, VrfPortInfo},
    vxlan::VxlanInfo,
    wifi::WifiInfo,
    xfrm::XfrmInfo,
};
pub(crate) use self::{
    iface::resolve_iface_index,
    inter_ifaces::{get_iface_name2index, get_ifaces, get_ifaces_with_handle},
    ip::{is_ipv6_addr, parse_ip_addr_str, parse_ip_net_addr_str},
    mptcp::{get_mptcp, merge_mptcp_info},
    route::get_routes,
    route_rule::get_route_rules,
};
