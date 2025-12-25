// SPDX-License-Identifier: Apache-2.0

mod ip;
#[allow(dead_code)] // some nla::parse_xx functions might be unused
mod nla;

pub(crate) use crate::netlink::{ip::*, nla::*};
