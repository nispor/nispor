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
use log::warn;
use netlink_generic::{GenericNetlinkHeader, GenericNetlinkMessageBuffer};
use netlink_packet_core::{
    DecodeError, NetlinkDeserializable, NetlinkHeader, NetlinkPayload,
    NetlinkSerializable,
};
use netlink_packet_utils::{nla::NlasIterator, Emitable, Parseable};

use crate::{CoalesceAttr, EthtoolHeader, FeatureAttr, PauseAttr, RingAttr};

const ETHTOOL_MSG_PAUSE_GET: u8 = 21;
const ETHTOOL_MSG_PAUSE_GET_REPLY: u8 = 22;
const ETHTOOL_GENL_VERSION: u8 = 1;
const ETHTOOL_MSG_FEATURES_GET: u8 = 11;
const ETHTOOL_MSG_FEATURES_GET_REPLY: u8 = 11;
const ETHTOOL_MSG_COALESCE_GET: u8 = 19;
const ETHTOOL_MSG_COALESCE_GET_REPLY: u8 = 20;
const ETHTOOL_MSG_RINGS_GET: u8 = 15;
const ETHTOOL_MSG_RINGS_GET_REPLY: u8 = 16;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EthoolAttr {
    Pause(Vec<PauseAttr>),
    Feature(Vec<FeatureAttr>),
    Coalesce(Vec<CoalesceAttr>),
    Ring(Vec<RingAttr>),
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
                version: ETHTOOL_GENL_VERSION,
            },
            nlas,
        }
    }

    pub fn new_feature_get(
        message_type: u16,
        iface_name: Option<&str>,
    ) -> Self {
        let nlas = match iface_name {
            // using ETHTOOL_FLAG_COMPACT_BITSETS, the netlink package
            // could be much smaller(without human readable string in it).
            // But we don't have good way converting these bites to human
            // readable strings, so we ask kernel to provide such string and
            // hope this does not cost us too much.
            Some(s) => EthoolAttr::Feature(vec![FeatureAttr::Header(vec![
                EthtoolHeader::DevName(s.to_string()),
            ])]),
            None => EthoolAttr::Feature(vec![FeatureAttr::Header(vec![])]),
        };
        EthtoolMessage {
            message_type,
            header: GenericNetlinkHeader {
                cmd: ETHTOOL_MSG_FEATURES_GET,
                version: ETHTOOL_GENL_VERSION,
            },
            nlas,
        }
    }

    pub fn new_coalesce_get(
        message_type: u16,
        iface_name: Option<&str>,
    ) -> Self {
        let nlas = match iface_name {
            Some(s) => EthoolAttr::Coalesce(vec![CoalesceAttr::Header(vec![
                EthtoolHeader::DevName(s.to_string()),
            ])]),
            None => EthoolAttr::Coalesce(vec![CoalesceAttr::Header(vec![])]),
        };
        EthtoolMessage {
            message_type,
            header: GenericNetlinkHeader {
                cmd: ETHTOOL_MSG_COALESCE_GET,
                version: ETHTOOL_GENL_VERSION,
            },
            nlas,
        }
    }

    pub fn new_ring_get(message_type: u16, iface_name: Option<&str>) -> Self {
        let nlas = match iface_name {
            Some(s) => EthoolAttr::Ring(vec![RingAttr::Header(vec![
                EthtoolHeader::DevName(s.to_string()),
            ])]),
            None => EthoolAttr::Ring(vec![RingAttr::Header(vec![])]),
        };
        EthtoolMessage {
            message_type,
            header: GenericNetlinkHeader {
                cmd: ETHTOOL_MSG_RINGS_GET,
                version: ETHTOOL_GENL_VERSION,
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
                EthoolAttr::Feature(nlas) => nlas.as_slice().buffer_len(),
                EthoolAttr::Coalesce(nlas) => nlas.as_slice().buffer_len(),
                EthoolAttr::Ring(nlas) => nlas.as_slice().buffer_len(),
            }
    }

    fn emit(&self, buffer: &mut [u8]) {
        self.header.emit(buffer);
        match &self.nlas {
            EthoolAttr::Pause(nlas) => nlas
                .as_slice()
                .emit(&mut buffer[self.header.buffer_len()..]),
            EthoolAttr::Feature(nlas) => nlas
                .as_slice()
                .emit(&mut buffer[self.header.buffer_len()..]),
            EthoolAttr::Coalesce(nlas) => nlas
                .as_slice()
                .emit(&mut buffer[self.header.buffer_len()..]),
            EthoolAttr::Ring(nlas) => nlas
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
            ETHTOOL_MSG_FEATURES_GET_REPLY => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse ethtool message attributes";
                for nla in NlasIterator::new(buf.payload()) {
                    let nla = &nla.context(error_msg)?;
                    let parsed = FeatureAttr::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                EthoolAttr::Feature(nlas)
            }
            ETHTOOL_MSG_COALESCE_GET_REPLY => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse ethtool message attributes";
                for nla in NlasIterator::new(buf.payload()) {
                    let nla = &nla.context(error_msg)?;
                    let parsed = CoalesceAttr::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                EthoolAttr::Coalesce(nlas)
            }
            ETHTOOL_MSG_RINGS_GET_REPLY => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse ethtool message attributes";
                for nla in NlasIterator::new(buf.payload()) {
                    let nla = &nla.context(error_msg)?;
                    let parsed = RingAttr::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                EthoolAttr::Ring(nlas)
            }
            _ => {
                warn!(
                    "ERR: Unsupported EthtoolMessage cmd {} payload {:?}",
                    header.cmd, payload
                );
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
