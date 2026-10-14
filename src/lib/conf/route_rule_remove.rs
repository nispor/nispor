// SPDX-License-Identifier: Apache-2.0

use std::{collections::BTreeMap, convert::Infallible};

use futures::{
    Stream, StreamExt,
    future::{Either, select},
};
use rtnetlink::{
    packet_core::{
        DecodeError, Emitable, NLM_F_ACK, NLM_F_CREATE, NLM_F_DUMP,
        NLM_F_DUMP_INTR, NLM_F_REQUEST, NetlinkDeserializable, NetlinkHeader,
        NetlinkMessage, NetlinkPayload, NetlinkSerializable, NlaBuffer,
        Parseable,
    },
    packet_route::rule::{RuleAttribute, RuleFlags, RuleHeader, RuleMessage},
    proto::{ConnectionHandle, new_connection},
    sys::{SocketAddr, protocols::NETLINK_ROUTE},
};
use serde::Serialize;

use crate::{ErrorKind, NisporError, RouteRule, query::get_rule};

/// Failure to remove routing rules, distinguishing safe permission fallback.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub enum RouteRuleRemoveError {
    /// The kernel denied a mutation before any mutation succeeded.
    PermissionDeniedBeforeChange(NisporError),
    /// The operation failed and may have partially changed routing rules.
    Failed(NisporError),
}

impl std::fmt::Display for RouteRuleRemoveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PermissionDeniedBeforeChange(error) | Self::Failed(error) => {
                error.fmt(f)
            }
        }
    }
}

impl std::error::Error for RouteRuleRemoveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PermissionDeniedBeforeChange(error) | Self::Failed(error) => {
                Some(error)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawRuleMessage {
    message_type: u16,
    payload: Vec<u8>,
}

// The generic netlink codec skips messages whose inner decoder fails. Keep
// every payload here and validate it before any mutation.
impl NetlinkDeserializable for RawRuleMessage {
    type Error = Infallible;

    fn deserialize(
        header: &NetlinkHeader,
        payload: &[u8],
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            message_type: header.message_type,
            payload: payload.to_vec(),
        })
    }
}

impl NetlinkSerializable for RawRuleMessage {
    fn message_type(&self) -> u16 {
        self.message_type
    }

    fn buffer_len(&self) -> usize {
        self.payload.len()
    }

    fn serialize(&self, buffer: &mut [u8]) {
        buffer.copy_from_slice(&self.payload);
    }
}

impl RawRuleMessage {
    fn parse(&self) -> Result<RuleMessage, DecodeError> {
        let rule = RuleMessage::parse(&self.payload)?;
        let mut offset = rule.header.buffer_len();
        while offset < self.payload.len() {
            let attribute = NlaBuffer::new_checked(&self.payload[offset..])?;
            let length = (usize::from(attribute.length()) + 3) & !3;
            if length > self.payload.len() - offset {
                return Err("truncated routing rule attribute padding".into());
            }
            offset += length;
        }
        Ok(rule)
    }

    fn normalize_for_replay(
        &mut self,
        header: &RuleHeader,
        missing_priority: bool,
    ) {
        let mut flags = header.flags;
        flags.remove(
            RuleFlags::Unresolved
                | RuleFlags::IifDetached
                | RuleFlags::OifDetached,
        );
        self.payload[8..12].copy_from_slice(&flags.bits().to_ne_bytes());
        if missing_priority {
            // Dumps omit priority zero; replay must select it explicitly.
            let priority = RuleAttribute::Priority(0);
            let offset = self.payload.len();
            self.payload.resize(offset + priority.buffer_len(), 0);
            priority.emit(&mut self.payload[offset..]);
        }
    }

    fn into_netlink(self) -> NetlinkMessage<Self> {
        NetlinkMessage::new(
            NetlinkHeader::default(),
            NetlinkPayload::InnerMessage(self),
        )
    }
}

/// Remove rules selected from a complete IPv4 and IPv6 snapshot.
///
/// Unselected records retain their selectors, duplicate counts and order.
/// Serialize other rule writers with this operation and provide rollback if
/// partial changes must be undone. Dropping the future closes its socket;
/// requests already submitted to the kernel may still have taken effect.
pub async fn remove_route_rules<F>(
    selector: F,
) -> Result<(), RouteRuleRemoveError>
where
    F: FnMut(&RouteRule) -> bool,
{
    let (mut connection, handle, _) =
        new_connection::<RawRuleMessage>(NETLINK_ROUTE)
            .map_err(rule_netlink_error)?;
    connection.set_forward_done(true);
    connection.set_forward_ack(true);
    let removal =
        std::pin::pin!(remove_route_rules_with_handle(&handle, selector,));
    // Pausing or cancelling cleanup also stops queued socket requests.
    match select(removal, connection).await {
        Either::Left((result, _)) => result,
        Either::Right(_) => {
            Err(rule_netlink_error("kernel rule connection closed"))
        }
    }
}

async fn remove_route_rules_with_handle<F>(
    handle: &ConnectionHandle<RawRuleMessage>,
    mut selector: F,
) -> Result<(), RouteRuleRemoveError>
where
    F: FnMut(&RouteRule) -> bool,
{
    let mut groups: BTreeMap<(u8, u32), Vec<(RawRuleMessage, bool)>> =
        BTreeMap::new();
    for family in [
        rtnetlink::packet_route::AddressFamily::Inet,
        rtnetlink::packet_route::AddressFamily::Inet6,
    ] {
        let header = RuleHeader {
            family,
            ..Default::default()
        };
        let mut payload = vec![0; header.buffer_len()];
        header.emit(&mut payload);
        let mut request = RawRuleMessage {
            message_type: libc::RTM_GETRULE,
            payload,
        }
        .into_netlink();
        request.header.flags = NLM_F_REQUEST | NLM_F_DUMP;
        let mut responses = handle
            .request(request, SocketAddr::new(0, 0))
            .map_err(rule_netlink_error)?;
        let mut done = false;
        while let Some(response) = responses.next().await {
            if response.header.flags & NLM_F_DUMP_INTR != 0 {
                return Err(rule_netlink_error(
                    "kernel rule dump was interrupted",
                ));
            }
            let mut raw = match response.payload {
                NetlinkPayload::InnerMessage(raw)
                    if raw.message_type == libc::RTM_NEWRULE =>
                {
                    raw
                }
                NetlinkPayload::Done(message) if message.code == 0 => {
                    done = true;
                    continue;
                }
                NetlinkPayload::Error(error) => {
                    return Err(rule_netlink_error(error));
                }
                _ => {
                    return Err(rule_netlink_error(
                        "invalid kernel rule dump response",
                    ));
                }
            };
            let projected = prepare_rule(&mut raw)?;
            if projected.address_family != family.into() {
                return Err(rule_netlink_error(
                    "unexpected family in kernel rule dump",
                ));
            }
            let key = (u8::from(family), projected.priority.unwrap_or(0));
            let remove = selector(&projected);
            groups.entry(key).or_default().push((raw, remove));
        }
        if !done {
            return Err(rule_netlink_error("incomplete kernel rule dump"));
        }
    }
    let mut changed = false;
    for rules in groups.values() {
        if !rules.iter().any(|(_, remove)| *remove) {
            continue;
        }
        // Zero selectors act as wildcards in kernel deletion. Rebuild the
        // priority group in order to preserve every unselected record.
        for (rule, _) in rules {
            if let Err(error) = update_kernel_rule(handle, rule, false).await {
                if let rtnetlink::Error::NetlinkError(ref error) = error
                    && error.to_io().raw_os_error() == Some(libc::ENOENT)
                {
                    continue;
                }
                return Err(mutation_error(error, changed));
            }
            changed = true;
        }
        for (rule, _) in rules.iter().filter(|(_, remove)| !remove) {
            update_kernel_rule(handle, rule, true)
                .await
                .map_err(|error| mutation_error(error, changed))?;
            changed = true;
        }
    }
    Ok(())
}

fn prepare_rule(
    raw: &mut RawRuleMessage,
) -> Result<RouteRule, RouteRuleRemoveError> {
    let mut rule = raw.parse().map_err(rule_netlink_error)?;
    let missing_priority = !rule
        .attributes
        .iter()
        .any(|attribute| matches!(attribute, RuleAttribute::Priority(_)));
    if missing_priority {
        rule.attributes.push(RuleAttribute::Priority(0));
    }
    raw.normalize_for_replay(&rule.header, missing_priority);
    get_rule(rule).map_err(RouteRuleRemoveError::Failed)
}

async fn update_kernel_rule(
    handle: &ConnectionHandle<RawRuleMessage>,
    rule: &RawRuleMessage,
    add: bool,
) -> Result<(), rtnetlink::Error> {
    let mut raw = rule.clone();
    raw.message_type = if add {
        libc::RTM_NEWRULE
    } else {
        libc::RTM_DELRULE
    };
    let mut request = raw.into_netlink();
    // CREATE without EXCL preserves duplicate rule records.
    request.header.flags =
        NLM_F_REQUEST | NLM_F_ACK | if add { NLM_F_CREATE } else { 0 };
    let responses = handle
        .request(request, SocketAddr::new(0, 0))
        .map_err(|_| rtnetlink::Error::RequestFailed)?;
    check_acknowledgement(responses).await
}

async fn check_acknowledgement<S>(
    mut responses: S,
) -> Result<(), rtnetlink::Error>
where
    S: Stream<Item = NetlinkMessage<RawRuleMessage>> + Unpin,
{
    let mut acknowledged = false;
    while let Some(response) = responses.next().await {
        if let NetlinkPayload::Error(error) = response.payload {
            if error.code.is_some() {
                return Err(rtnetlink::Error::NetlinkError(error));
            }
            acknowledged = true;
        } else {
            return Err(rtnetlink::Error::RequestFailed);
        }
    }
    if acknowledged {
        Ok(())
    } else {
        Err(rtnetlink::Error::RequestFailed)
    }
}

fn rule_netlink_error(error: impl std::fmt::Display) -> RouteRuleRemoveError {
    RouteRuleRemoveError::Failed(NisporError::new(
        ErrorKind::NetlinkError,
        format!("Failed to update kernel route rules: {error}"),
    ))
}

fn mutation_error(
    error: rtnetlink::Error,
    changed: bool,
) -> RouteRuleRemoveError {
    if let rtnetlink::Error::NetlinkError(ref netlink_error) = error
        && matches!(
            netlink_error.to_io().raw_os_error(),
            Some(libc::EPERM | libc::EACCES)
        )
    {
        let error = NisporError::permission_deny(error.to_string());
        if !changed {
            return RouteRuleRemoveError::PermissionDeniedBeforeChange(error);
        }
        return RouteRuleRemoveError::Failed(error);
    }
    rule_netlink_error(error)
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroI32;

    use futures::{executor::block_on, stream};
    use rtnetlink::{
        packet_core::{DefaultNla, ErrorMessage},
        packet_route::{AddressFamily, rule::RuleAction},
    };

    use super::*;

    fn raw_rule(family: AddressFamily) -> RawRuleMessage {
        let mut rule = RuleMessage::default();
        rule.header.family = family;
        rule.header.table = 10;
        rule.header.action = RuleAction::ToTable;
        rule.attributes = vec![RuleAttribute::TunId(0x1234_5678_9abc_def0)];
        let mut payload = vec![0; rule.buffer_len()];
        rule.emit(&mut payload);
        let unknown = DefaultNla::new(0x8000 | 4000, vec![0xab]);
        let offset = payload.len();
        payload.resize(offset + unknown.buffer_len(), 0);
        unknown.emit(&mut payload[offset..]);
        *payload.last_mut().unwrap() = 0xcd;
        RawRuleMessage {
            message_type: libc::RTM_NEWRULE,
            payload,
        }
    }

    fn error_ack(code: Option<i32>) -> NetlinkMessage<RawRuleMessage> {
        let mut error = ErrorMessage::default();
        error.code = code.and_then(NonZeroI32::new);
        NetlinkMessage::new(
            NetlinkHeader::default(),
            NetlinkPayload::Error(error),
        )
    }

    #[test]
    fn raw_rule_roundtrip_preserves_unknown_attributes_and_padding() {
        for family in [AddressFamily::Inet, AddressFamily::Inet6] {
            let raw = raw_rule(family);
            assert_eq!(
                get_rule(raw.parse().unwrap()).unwrap().tun_id,
                Some(0x1234_5678_9abc_def0)
            );
            let mut message = raw.clone().into_netlink();
            message.finalize();
            let mut bytes = vec![0; message.buffer_len()];
            message.serialize(&mut bytes);
            let decoded =
                NetlinkMessage::<RawRuleMessage>::deserialize(&bytes).unwrap();
            assert_eq!(decoded.payload, NetlinkPayload::InnerMessage(raw));
        }
    }

    #[test]
    fn zero_priority_is_explicit_before_selection_and_replay() {
        let mut raw = raw_rule(AddressFamily::Inet);
        let retained = RuleFlags::Invert | RuleFlags::from_bits_retain(1 << 31);
        let flags = retained
            | RuleFlags::Unresolved
            | RuleFlags::IifDetached
            | RuleFlags::OifDetached;
        raw.payload[8..12].copy_from_slice(&flags.bits().to_ne_bytes());
        let original = raw.payload.clone();

        let projected = prepare_rule(&mut raw).unwrap();
        assert_eq!(projected.priority, Some(0));
        assert_eq!(projected.tun_id, Some(0x1234_5678_9abc_def0));
        assert_eq!(&raw.payload[..8], &original[..8]);
        assert_eq!(&raw.payload[12..original.len()], &original[12..]);
        let parsed = raw.parse().unwrap();
        assert_eq!(parsed.header.flags, retained);
        assert_eq!(parsed.attributes.last(), Some(&RuleAttribute::Priority(0)));

        let normalized = raw.clone();
        prepare_rule(&mut raw).unwrap();
        assert_eq!(raw, normalized);
    }

    #[test]
    fn malformed_rule_is_captured_and_rejected_explicitly() {
        let valid = raw_rule(AddressFamily::Inet);
        let mut missing_padding = valid.payload.clone();
        missing_padding.pop();
        let mut invalid_priority = valid.payload[..12].to_vec();
        invalid_priority.extend_from_slice(&6_u16.to_ne_bytes());
        invalid_priority.extend_from_slice(&6_u16.to_ne_bytes());
        invalid_priority.extend_from_slice(&[0; 4]);

        for payload in [vec![0; 11], missing_padding, invalid_priority] {
            let mut raw = RawRuleMessage::deserialize(
                &NetlinkHeader::default(),
                &payload,
            )
            .unwrap();
            assert_eq!(raw.payload, payload);
            assert!(matches!(
                prepare_rule(&mut raw),
                Err(RouteRuleRemoveError::Failed(_))
            ));
        }
    }

    #[test]
    fn permission_fallback_requires_explicit_denial_before_change() {
        for code in [libc::EPERM, libc::EACCES] {
            let NetlinkPayload::Error(error) = error_ack(Some(-code)).payload
            else {
                unreachable!();
            };
            assert!(matches!(
                mutation_error(
                    rtnetlink::Error::NetlinkError(error.clone()),
                    false
                ),
                RouteRuleRemoveError::PermissionDeniedBeforeChange(_)
            ));
            assert!(matches!(
                mutation_error(rtnetlink::Error::NetlinkError(error), true),
                RouteRuleRemoveError::Failed(_)
            ));
        }
        assert!(matches!(
            mutation_error(rtnetlink::Error::RequestFailed, false),
            RouteRuleRemoveError::Failed(_)
        ));
        let NetlinkPayload::Error(error) =
            error_ack(Some(-libc::ENOENT)).payload
        else {
            unreachable!();
        };
        assert!(matches!(
            mutation_error(rtnetlink::Error::NetlinkError(error), false),
            RouteRuleRemoveError::Failed(_)
        ));
    }

    #[test]
    fn mutation_requires_successful_acknowledgement() {
        assert!(
            block_on(check_acknowledgement(stream::iter(vec![error_ack(
                None
            )])))
            .is_ok()
        );
        for responses in [
            vec![],
            vec![raw_rule(AddressFamily::Inet).into_netlink()],
            vec![error_ack(Some(-libc::EPERM))],
        ] {
            assert!(
                block_on(check_acknowledgement(stream::iter(responses)))
                    .is_err()
            );
        }
    }
}
