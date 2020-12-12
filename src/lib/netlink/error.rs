use crate::NisporError;
use libc::{EEXIST, EPERM};
use rtnetlink::packet::ErrorMessage;

pub(crate) fn parse_apply_netlink_error(
    netlink_err: &ErrorMessage,
) -> NisporError {
    match netlink_err.code.abs() {
        EEXIST => NisporError::bug(&format!(
            "Got netlink EEXIST error: {}",
            netlink_err
        )),
        EPERM => NisporError::permission_deny(&format!("{}", netlink_err,)),
        _ => NisporError::bug(&format!(
            "Got netlink unknown error: code {}, msg: {}",
            netlink_err.code, netlink_err,
        )),
    }
}
