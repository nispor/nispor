// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod bond;
mod bridge;
mod ethtool;
mod iface;
mod ifaces;
mod mac_vlan;
mod mac_vtap;
mod sriov;
mod tun;
mod veth;
mod vlan;
mod vrf;
mod vxlan;

pub use crate::ifaces::bond::BondAdInfo;
pub use crate::ifaces::bond::BondAdSelect;
pub use crate::ifaces::bond::BondAllSubordinatesActive;
pub use crate::ifaces::bond::BondArpValidate;
pub use crate::ifaces::bond::BondFailOverMac;
pub use crate::ifaces::bond::BondInfo;
pub use crate::ifaces::bond::BondLacpRate;
pub use crate::ifaces::bond::BondMiiStatus;
pub use crate::ifaces::bond::BondMode;
pub use crate::ifaces::bond::BondModeArpAllTargets;
pub use crate::ifaces::bond::BondPrimaryReselect;
pub use crate::ifaces::bond::BondSubordinateInfo;
pub use crate::ifaces::bond::BondSubordinateState;
pub use crate::ifaces::bond::BondXmitHashPolicy;
pub use crate::ifaces::bridge::BridgeInfo;
pub use crate::ifaces::bridge::BridgePortInfo;
pub use crate::ifaces::bridge::BridgeVlanEntry;
pub use crate::ifaces::ethtool::{EthtoolInfo, EthtoolPauseInfo};
pub(crate) use crate::ifaces::iface::get_iface_name_by_index;
pub use crate::ifaces::iface::ControllerType;
pub use crate::ifaces::iface::Iface;
pub use crate::ifaces::iface::IfaceConf;
pub use crate::ifaces::iface::IfaceState;
pub use crate::ifaces::iface::IfaceType;
pub(crate) use crate::ifaces::ifaces::get_ifaces;
pub use crate::ifaces::mac_vlan::MacVlanInfo;
pub use crate::ifaces::mac_vtap::MacVtapInfo;
pub use crate::ifaces::sriov::SriovInfo;
pub use crate::ifaces::tun::TunInfo;
pub use crate::ifaces::veth::VethInfo;
pub use crate::ifaces::vlan::VlanInfo;
pub use crate::ifaces::vlan::VlanProtocol;
pub use crate::ifaces::vrf::VrfInfo;
pub use crate::ifaces::vrf::VrfSubordinateInfo;
pub use crate::ifaces::vxlan::VxlanInfo;
