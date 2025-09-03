// SPDX-License-Identifier: Apache-2.0

mod bridge;
mod bridge_vlan;
mod ip;
#[allow(dead_code)] // some nla::parse_xx functions might be unused
mod nla;

pub(crate) use crate::netlink::{bridge::*, bridge_vlan::*, ip::*, nla::*};
