// SPDX-License-Identifier: Apache-2.0

mod alt_name;
mod base_iface;
mod bond;
mod bond_port;
mod bridge;
mod bridge_port;
mod bridge_vlan;
mod dummy;
mod iface;
mod inter_ifaces;
mod ip;
mod route;
mod veth;
mod vlan;

pub use self::{
    alt_name::AltNameConf,
    bond::BondConf,
    bond_port::BondPortConf,
    bridge::BridgeConf,
    bridge_port::BridgePortConf,
    dummy::DummyConf,
    iface::IfaceConf,
    ip::{IpAddrConf, IpConf},
    route::RouteConf,
    veth::VethConf,
    vlan::VlanConf,
};
pub(crate) use self::{
    inter_ifaces::apply_ifaces_conf, route::apply_routes_conf,
};
