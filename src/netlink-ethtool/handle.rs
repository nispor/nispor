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

use futures::Stream;
use netlink_packet_core::NetlinkMessage;
use netlink_proto::{sys::SocketAddr, ConnectionHandle};

use crate::{EthtoolError, EthtoolMessage, FeatureHandle, PauseHandle};

#[derive(Clone, Debug)]
pub struct EthtoolHandle {
    pub family_id: u16,
    pub handle: ConnectionHandle<EthtoolMessage>,
}

impl EthtoolHandle {
    pub(crate) fn new(
        handle: ConnectionHandle<EthtoolMessage>,
        family_id: u16,
    ) -> Self {
        EthtoolHandle { family_id, handle }
    }

    pub fn pause(&mut self) -> PauseHandle {
        PauseHandle::new(self.clone())
    }

    pub fn feature(&mut self) -> FeatureHandle {
        FeatureHandle::new(self.clone())
    }

    pub fn request(
        &mut self,
        message: NetlinkMessage<EthtoolMessage>,
    ) -> Result<impl Stream<Item = NetlinkMessage<EthtoolMessage>>, EthtoolError>
    {
        self.handle
            .request(message, SocketAddr::new(0, 0))
            .map_err(|e| {
                EthtoolError::RequestFailed(format!(
                    "BUG: Request failed with {}",
                    e
                ))
            })
    }
}
