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

#[macro_export]
macro_rules! try_genl {
    ($msg: expr) => {{
        use netlink_packet_core::{NetlinkMessage, NetlinkPayload};
        use $crate::GenericNetlinkError;

        let (header, payload) = $msg.into_parts();
        match payload {
            NetlinkPayload::InnerMessage(msg) => msg,
            NetlinkPayload::Error(err) => {
                return Err(GenericNetlinkError::NetlinkError(err))
            }
            _ => {
                return Err(GenericNetlinkError::UnexpectedMessage(
                    NetlinkMessage::new(header, payload),
                ))
            }
        }
    }};
}
