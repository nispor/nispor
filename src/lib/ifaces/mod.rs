// SPDX-License-Identifier: Apache-2.0

mod bond;
mod bridge;
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
mod sriov;
mod tun;
mod veth;
mod vlan;
mod vrf;
mod vxlan;

pub use crate::ifaces::bond::*;
pub use crate::ifaces::bridge::*;
pub use crate::ifaces::ethtool::*;
pub use crate::ifaces::iface::*;
pub use crate::ifaces::ipoib::{IpoibInfo, IpoibMode};
pub use crate::ifaces::mac_vlan::*;
pub use crate::ifaces::mac_vtap::*;
pub use crate::ifaces::macsec::*;
pub use crate::ifaces::sriov::*;
pub use crate::ifaces::tun::*;
pub use crate::ifaces::veth::*;
pub use crate::ifaces::vlan::*;
pub use crate::ifaces::vrf::*;
pub use crate::ifaces::vxlan::*;

pub(crate) use crate::ifaces::inter_ifaces::{
    change_ifaces, create_ifaces, delete_ifaces, get_iface_name2index,
    get_ifaces,
};
