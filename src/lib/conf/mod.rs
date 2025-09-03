// SPDX-License-Identifier: Apache-2.0

mod alt_name;
mod bond;
mod bridge;
mod iface;
mod inter_ifaces;
mod ip;
mod route;
mod veth;
mod vlan;

pub use self::{
    alt_name::AltNameConf,
    bond::BondConf,
    bridge::BridgeConf,
    iface::IfaceConf,
    ip::{IpAddrConf, IpConf},
    route::RouteConf,
    veth::VethConf,
    vlan::VlanConf,
};
pub(crate) use self::{
    inter_ifaces::{change_ifaces, create_ifaces, delete_ifaces},
    route::apply_routes_conf,
};
