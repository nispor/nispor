// SPDX-License-Identifier: Apache-2.0

use std::os::unix::io::RawFd;

use crate::{
    NetStateIfaceFilter, NetStateRouteFilter, NetStateRouteRuleFilter,
    NisporError,
};

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
/// The `NetStateFilter::default()` will retrieve full information.
/// To query only the interested part, please use `NetStateFilter::minimum()`
/// with proper sub-filter set with `Some()`.
pub struct NetStateFilter {
    /// Filter applied to interfaces, default is NetStateIfaceFilter::default()
    /// -- all interface with full information.
    /// When set to None, no interface will be included in result.
    pub iface: Option<NetStateIfaceFilter>,
    /// Filter applied to route entries, default is
    /// NetStateRouteFilter::default() -- full routes information.
    /// When set to None, no route will be included in result.
    pub route: Option<NetStateRouteFilter>,

    /// Filter applied to route rule entries, default is
    /// NetStateRouteRuleFilter::default() -- full route rule infromation.
    /// When set to None, no route rule will be included in result.
    pub route_rule: Option<NetStateRouteRuleFilter>,
}

impl Default for NetStateFilter {
    fn default() -> Self {
        Self {
            iface: Some(NetStateIfaceFilter::default()),
            route: Some(NetStateRouteFilter::default()),
            route_rule: Some(NetStateRouteRuleFilter::default()),
        }
    }
}

impl NetStateFilter {
    /// Return a filter excluding all information.
    /// Could be used to query interested information only by setting
    /// the sub-filter to `Some()`.
    pub fn minimum() -> Self {
        Self {
            iface: None,
            route: None,
            route_rule: None,
        }
    }
}

const NETLINK_GET_STRICT_CHK: i32 = 12;

pub(crate) fn enable_kernel_strict_check(fd: RawFd) -> Result<(), NisporError> {
    if unsafe {
        libc::setsockopt(
            fd,
            libc::SOL_NETLINK,
            NETLINK_GET_STRICT_CHK,
            1u32.to_ne_bytes().as_ptr() as *const _,
            4,
        )
    } != 0
    {
        let e = NisporError::bug(format!(
            "Failed to set socket option NETLINK_GET_STRICT_CHK: error {}",
            std::io::Error::last_os_error()
        ));
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}
