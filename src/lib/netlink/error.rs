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

use crate::NisporError;
use libc::{EEXIST, EPERM};
use rtnetlink::packet::ErrorMessage;

pub(crate) fn parse_apply_netlink_error(
    netlink_err: &ErrorMessage,
) -> NisporError {
    match netlink_err.code.abs() {
        EEXIST => NisporError::bug(format!(
            "Got netlink EEXIST error: {}",
            netlink_err
        )),
        EPERM => NisporError::permission_deny(format!("{}", netlink_err,)),
        _ => NisporError::bug(format!(
            "Got netlink unknown error: code {}, msg: {}",
            netlink_err.code, netlink_err,
        )),
    }
}
