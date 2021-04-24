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

use netlink_packet_utils::{
    buffer, buffer_check_length, buffer_common, fields, getter, setter,
    DecodeError,
};

pub(crate) const GENL_HEADER_LEN: usize = 4;
pub const GENL_ID_CTRL: u16 = 0x10;

buffer!(GenericNetlinkMessageBuffer(GENL_HEADER_LEN) {
    cmd: (u8, 0),
    version: (u8, 1),
    reserve_1: (u8, 2),
    payload: (slice, GENL_HEADER_LEN..),
});
