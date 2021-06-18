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

mod error;
mod ifaces;
mod ip;
mod mac;
mod net_conf;
mod net_state;
mod netlink;
mod route;
mod route_rule;

pub use crate::error::NisporError;
pub use crate::ifaces::BondAdInfo;
pub use crate::ifaces::BondAdSelect;
pub use crate::ifaces::BondAllSubordinatesActive;
pub use crate::ifaces::BondArpValidate;
pub use crate::ifaces::BondFailOverMac;
pub use crate::ifaces::BondInfo;
pub use crate::ifaces::BondLacpRate;
pub use crate::ifaces::BondMiiStatus;
pub use crate::ifaces::BondMode;
pub use crate::ifaces::BondModeArpAllTargets;
pub use crate::ifaces::BondPrimaryReselect;
pub use crate::ifaces::BondSubordinateInfo;
pub use crate::ifaces::BondSubordinateState;
pub use crate::ifaces::BondXmitHashPolicy;
pub use crate::ifaces::BridgeInfo;
pub use crate::ifaces::BridgePortInfo;
pub use crate::ifaces::BridgeVlanEntry;
pub use crate::ifaces::ControllerType;
pub use crate::ifaces::Iface;
pub use crate::ifaces::IfaceFlags;
pub use crate::ifaces::IfaceState;
pub use crate::ifaces::IfaceType;
pub use crate::ifaces::VlanInfo;
pub use crate::ifaces::VlanProtocol;
pub use crate::ip::IpAddrConf;
pub use crate::ip::IpConf;
pub use crate::ip::IpFamily;
pub use crate::ip::Ipv4AddrInfo;
pub use crate::ip::Ipv4Info;
pub use crate::ip::Ipv6AddrInfo;
pub use crate::ip::Ipv6Info;
pub(crate) use crate::mac::parse_as_mac;
pub use crate::net_conf::NetConf;
pub use crate::net_state::NetState;
pub use crate::route::AddressFamily;
pub use crate::route::MultipathRoute;
pub use crate::route::MultipathRouteFlags;
pub use crate::route::Route;
pub use crate::route::RouteProtocol;
pub use crate::route::RouteScope;
pub use crate::route::RouteType;
pub use crate::route_rule::RouteRule;
