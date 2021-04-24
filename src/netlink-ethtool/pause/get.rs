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

use futures::{self, future::Either, FutureExt, StreamExt, TryStream};
use netlink_packet_core::{NetlinkMessage, NLM_F_DUMP, NLM_F_REQUEST};

use crate::{try_ethtool, EthtoolError, EthtoolHandle, EthtoolMessage};

pub struct PauseGetRequest {
    handle: EthtoolHandle,
    iface_name: Option<String>,
}

impl PauseGetRequest {
    pub(crate) fn new(handle: EthtoolHandle, iface_name: Option<&str>) -> Self {
        PauseGetRequest {
            handle,
            iface_name: iface_name.map(|i| i.to_string()),
        }
    }

    pub fn execute(
        self,
    ) -> impl TryStream<Ok = EthtoolMessage, Error = EthtoolError> {
        let PauseGetRequest {
            mut handle,
            iface_name,
        } = self;

        let nl_header_flags = match iface_name {
            None => NLM_F_DUMP | NLM_F_REQUEST,
            Some(_) => NLM_F_REQUEST,
        };

        let ethtool_msg = EthtoolMessage::new_pause_get(
            handle.family_id,
            iface_name.as_deref(),
        );

        let mut nl_msg = NetlinkMessage::from(ethtool_msg);

        nl_msg.header.flags = nl_header_flags;

        match handle.request(nl_msg) {
            Ok(response) => {
                Either::Left(response.map(move |msg| Ok(try_ethtool!(msg))))
            }
            Err(e) => Either::Right(
                futures::future::err::<EthtoolMessage, EthtoolError>(e)
                    .into_stream(),
            ),
        }
    }
}
