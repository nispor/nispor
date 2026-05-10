// SPDX-License-Identifier: Apache-2.0

mod utils;

mod alt_name;
mod base_info;
mod bond;
mod bridge;
mod bridge_vlan_filter;
mod dummy;
mod ethtool;
mod gre;
mod hsr;
mod ip;
mod ip_vlan;
mod iptunnel;
mod mac_vlan;
mod mac_vtap;
mod macsec;
mod route;
mod route_rule;
mod tap;
mod tun;
mod veth;
mod vlan;
mod vrf;
mod vxlan;
#[cfg(feature = "wireguard")]
mod wireguard;
mod xfrm;
