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

use std::io;

use futures::channel::mpsc::UnboundedReceiver;
use netlink_packet_core::NetlinkMessage;
use netlink_proto::{self, Connection};
use netlink_sys::{constants::NETLINK_GENERIC, SocketAddr};

use crate::{GenericNetlinkHandle, GenericNetlinkMessage};

#[allow(clippy::type_complexity)]
pub fn new_connection() -> io::Result<(
    Connection<GenericNetlinkMessage>,
    GenericNetlinkHandle,
    UnboundedReceiver<(NetlinkMessage<GenericNetlinkMessage>, SocketAddr)>,
)> {
    let (conn, handle, messages) =
        netlink_proto::new_connection(NETLINK_GENERIC)?;
    Ok((conn, GenericNetlinkHandle::new(handle), messages))
}
