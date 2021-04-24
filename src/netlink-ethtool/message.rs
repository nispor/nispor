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

use anyhow::Context;
use netlink_generic::{GenericNetlinkHeader, GenericNetlinkMessageBuffer};
use netlink_packet_core::{
    DecodeError, NetlinkDeserializable, NetlinkHeader, NetlinkPayload,
    NetlinkSerializable,
};
use netlink_packet_utils::{nla::NlasIterator, Emitable, Parseable};

use crate::{EthtoolHeader, PauseAttr};

const ETHTOOL_MSG_PAUSE_GET: u8 = 21;
const ETHTOOL_MSG_PAUSE_GET_REPLY: u8 = 22;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EthoolAttr {
    Pause(Vec<PauseAttr>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EthtoolMessage {
    pub message_type: u16,
    pub header: GenericNetlinkHeader,
    pub nlas: EthoolAttr,
}

impl EthtoolMessage {
    pub fn new_pause_get(message_type: u16, iface_name: Option<&str>) -> Self {
        let nlas = match iface_name {
            Some(s) => EthoolAttr::Pause(vec![PauseAttr::Header(vec![
                EthtoolHeader::DevName(s.to_string()),
            ])]),
            None => EthoolAttr::Pause(vec![PauseAttr::Header(vec![])]),
        };
        EthtoolMessage {
            message_type,
            header: GenericNetlinkHeader {
                cmd: ETHTOOL_MSG_PAUSE_GET,
                version: 1, // No idea the correct version.
            },
            nlas,
        }
    }
    fn message_type(&self) -> u16 {
        self.message_type
    }
}

impl Emitable for EthtoolMessage {
    fn buffer_len(&self) -> usize {
        self.header.buffer_len()
            + match &self.nlas {
                EthoolAttr::Pause(nlas) => nlas.as_slice().buffer_len(),
            }
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        match &self.nlas {
            EthoolAttr::Pause(nlas) => nlas
                .as_slice()
                .emit(&mut buffer[self.header.buffer_len()..]),
        };
    }
}

impl NetlinkSerializable<EthtoolMessage> for EthtoolMessage {
    fn message_type(&self) -> u16 {
        self.message_type()
    }

    fn buffer_len(&self) -> usize {
        <Self as Emitable>::buffer_len(self)
    }

    fn serialize(&self, buffer: &mut [u8]) {
        self.emit(buffer)
    }
}

impl NetlinkDeserializable<EthtoolMessage> for EthtoolMessage {
    type Error = DecodeError;
    fn deserialize(
        nl_header: &NetlinkHeader,
        payload: &[u8],
    ) -> Result<Self, Self::Error> {
        let buf = GenericNetlinkMessageBuffer::new(payload);
        let header = GenericNetlinkHeader::parse(&buf)
            .context("failed to parse generic netlink message header")?;

        let nlas = match header.cmd {
            ETHTOOL_MSG_PAUSE_GET_REPLY => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse ethtool message attributes";
                for nla in NlasIterator::new(buf.payload()) {
                    let nla = &nla.context(error_msg)?;
                    let parsed = PauseAttr::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                EthoolAttr::Pause(nlas)
            }
            _ => {
                return Err(format!("Unknown command {}", header.cmd).into());
            }
        };

        Ok(EthtoolMessage {
            message_type: nl_header.message_type,
            header,
            nlas,
        })
    }
}

impl From<EthtoolMessage> for NetlinkPayload<EthtoolMessage> {
    fn from(message: EthtoolMessage) -> Self {
        NetlinkPayload::InnerMessage(message)
    }
}
