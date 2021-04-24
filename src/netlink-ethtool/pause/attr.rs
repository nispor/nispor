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
use byteorder::{ByteOrder, NativeEndian};
use netlink_packet_utils::{
    nla::{self, DefaultNla, NlaBuffer, NlasIterator, NLA_F_NESTED},
    parsers::{parse_u64, parse_u8},
    DecodeError, Emitable, Parseable,
};

use crate::EthtoolHeader;

const ETHTOOL_A_PAUSE_HEADER: u16 = 1;
const ETHTOOL_A_PAUSE_AUTONEG: u16 = 2;
const ETHTOOL_A_PAUSE_RX: u16 = 3;
const ETHTOOL_A_PAUSE_TX: u16 = 4;
const ETHTOOL_A_PAUSE_STATS: u16 = 5;

const ETHTOOL_A_PAUSE_STAT_TX_FRAMES: u16 = 2;
const ETHTOOL_A_PAUSE_STAT_RX_FRAMES: u16 = 3;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PauseStatAttr {
    Rx(u64),
    Tx(u64),
    Other(DefaultNla),
}

impl nla::Nla for PauseStatAttr {
    fn value_len(&self) -> usize {
        match self {
            Self::Rx(_) | Self::Tx(_) => 8,
            Self::Other(attr) => attr.value_len(),
        }
    }

    fn kind(&self) -> u16 {
        match self {
            Self::Rx(_) => ETHTOOL_A_PAUSE_STAT_RX_FRAMES,
            Self::Tx(_) => ETHTOOL_A_PAUSE_STAT_RX_FRAMES,
            Self::Other(attr) => attr.kind(),
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        match self {
            Self::Rx(value) | Self::Tx(value) => {
                NativeEndian::write_u64(buffer, *value)
            }
            Self::Other(ref attr) => attr.emit_value(buffer),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>>
    for PauseStatAttr
{
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        let payload = buf.value();
        Ok(match buf.kind() {
            ETHTOOL_A_PAUSE_STAT_TX_FRAMES => Self::Tx(
                parse_u64(payload)
                    .context("invalid ETHTOOL_A_PAUSE_STAT_TX_FRAMES value")?,
            ),
            ETHTOOL_A_PAUSE_STAT_RX_FRAMES => Self::Rx(
                parse_u64(payload)
                    .context("invalid ETHTOOL_A_PAUSE_STAT_RX_FRAMES value")?,
            ),
            _ => Self::Other(
                DefaultNla::parse(buf).context("invalid NLA (unknown kind)")?,
            ),
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PauseAttr {
    Header(Vec<EthtoolHeader>),
    AutoNeg(bool),
    Rx(bool),
    Tx(bool),
    Stats(Vec<PauseStatAttr>),
    Other(DefaultNla),
}

impl nla::Nla for PauseAttr {
    fn value_len(&self) -> usize {
        match self {
            Self::Header(hdrs) => hdrs.as_slice().buffer_len(),
            Self::AutoNeg(_) | Self::Rx(_) | Self::Tx(_) => 1,
            Self::Stats(ref nlas) => nlas.as_slice().buffer_len(),
            Self::Other(attr) => attr.value_len(),
        }
    }

    fn kind(&self) -> u16 {
        match self {
            Self::Header(_) => ETHTOOL_A_PAUSE_HEADER | NLA_F_NESTED,
            Self::AutoNeg(_) => ETHTOOL_A_PAUSE_AUTONEG,
            Self::Rx(_) => ETHTOOL_A_PAUSE_RX,
            Self::Tx(_) => ETHTOOL_A_PAUSE_TX,
            Self::Stats(_) => ETHTOOL_A_PAUSE_STATS | NLA_F_NESTED,
            Self::Other(attr) => attr.kind(),
        }
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        match self {
            Self::Header(ref nlas) => nlas.as_slice().emit(buffer),
            Self::AutoNeg(value) | Self::Rx(value) | Self::Tx(value) => {
                buffer[0] = *value as u8
            }
            Self::Stats(ref nlas) => nlas.as_slice().emit(buffer),
            Self::Other(ref attr) => attr.emit(buffer),
        }
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Parseable<NlaBuffer<&'a T>> for PauseAttr {
    fn parse(buf: &NlaBuffer<&'a T>) -> Result<Self, DecodeError> {
        let payload = buf.value();
        Ok(match buf.kind() {
            ETHTOOL_A_PAUSE_HEADER => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse pause header attributes";
                for nla in NlasIterator::new(payload) {
                    let nla = &nla.context(error_msg)?;
                    let parsed =
                        EthtoolHeader::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                Self::Header(nlas)
            }
            ETHTOOL_A_PAUSE_AUTONEG => Self::AutoNeg(
                parse_u8(payload)
                    .context("invalid ETHTOOL_A_PAUSE_AUTONEG value")?
                    == 1,
            ),
            ETHTOOL_A_PAUSE_RX => Self::Rx(
                parse_u8(payload)
                    .context("invalid ETHTOOL_A_PAUSE_RX value")?
                    == 1,
            ),
            ETHTOOL_A_PAUSE_TX => Self::Tx(
                parse_u8(payload)
                    .context("invalid ETHTOOL_A_PAUSE_TX value")?
                    == 1,
            ),
            ETHTOOL_A_PAUSE_STATS => {
                let mut nlas = Vec::new();
                let error_msg = "failed to parse pause stats attributes";
                for nla in NlasIterator::new(payload) {
                    let nla = &nla.context(error_msg)?;
                    let parsed =
                        PauseStatAttr::parse(nla).context(error_msg)?;
                    nlas.push(parsed);
                }
                Self::Stats(nlas)
            }
            _ => Self::Other(
                DefaultNla::parse(buf).context("invalid NLA (unknown kind)")?,
            ),
        })
    }
}
