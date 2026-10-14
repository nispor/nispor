// SPDX-License-Identifier: Apache-2.0

use std::panic;

use super::utils::assert_value_match;
use crate::{NetConf, NetState, RouteRule};

const TEST_TABLE_ID: u32 = 100;
const TEST_RULE_PRIORITIES: [u32; 4] = [1000, 1001, 1002, 1003];

const EXPECTED_YAML_OUTPUT: &str = r#"---
- action: blackhole
  address-family: ipv6
  flags: []
  tos: 0
  table: 100
  dst: 2001:db8:f::252/128
  src: 2001:db8:f::255/128
  iif: eth1
  oif: eth2
  priority: 998
- action: table
  address-family: ipv6
  flags: []
  tos: 16
  table: 100
  dst: "2001:db8:f::253/128"
  src: "2001:db8:f::254/128"
  iif: eth1
  oif: eth2
  priority: 999
- action: unreachable
  address-family: ipv4
  flags: []
  tos: 0
  dst: 192.0.2.2/32
  src: 192.0.2.1/32
  iif: eth1
  oif: eth2
  priority: 998
- action: table
  address-family: ipv4
  flags: []
  tos: 16
  table: 100
  dst: 192.0.2.2/32
  src: 192.0.2.1/32
  iif: eth1
  oif: eth2
  priority: 999"#;

#[test]
fn test_get_route_rule_yaml() {
    with_route_rule_test_iface(|| {
        let state = NetState::retrieve().unwrap();
        let mut expected_rules = Vec::new();
        for mut rule in state.rules {
            if Some(TEST_TABLE_ID) == rule.table {
                // Travis CI Ubuntu 18.04 does not support protocol.
                rule.protocol = None;
                expected_rules.push(rule)
            }
        }
        assert_value_match(EXPECTED_YAML_OUTPUT, &expected_rules);
    });
}

const ADD_ROUTE_RULE_YML: &str = r#"---
rules:
- address-family: ipv4
  action: table
  src: 198.51.100.0/24
  dst: 192.0.2.0/24
  iif: eth1
  table: 200
  priority: 1000
- address-family: ipv6
  action: blackhole
  src: 2001:db8:f::/64
  priority: 1001
- address-family: ipv4
  action: goto
  goto: 1003
  priority: 1002
- address-family: ipv4
  action: blackhole
  priority: 1003"#;

const REMOVE_ROUTE_RULE_YML: &str = r#"---
rules:
- address-family: ipv4
  action: table
  src: 198.51.100.0/24
  dst: 192.0.2.0/24
  iif: eth1
  table: 200
  priority: 1000
  remove: true
- address-family: ipv6
  action: blackhole
  src: 2001:db8:f::/64
  priority: 1001
  remove: true
- address-family: ipv4
  action: goto
  goto: 1003
  priority: 1002
  remove: true
- address-family: ipv4
  action: blackhole
  priority: 1003
  remove: true"#;

const EXPECTED_ADDED_RULES_YAML: &str = r#"---
- action: table
  address-family: ipv4
  flags: []
  tos: 0
  table: 200
  src: 198.51.100.0/24
  dst: 192.0.2.0/24
  iif: eth1
  priority: 1000
- action: blackhole
  address-family: ipv6
  flags: []
  tos: 0
  src: 2001:db8:f::/64
  priority: 1001
- action: goto
  address-family: ipv4
  flags: []
  tos: 0
  goto: 1003
  priority: 1002
- action: blackhole
  address-family: ipv4
  flags: []
  tos: 0
  priority: 1003"#;

#[test]
fn test_add_remove_route_rule_yaml() {
    with_route_rule_test_iface(|| {
        let net_conf: NetConf =
            serde_yaml::from_str(ADD_ROUTE_RULE_YML).unwrap();
        net_conf.apply().unwrap();
        // Apply twice to test whether crate ignores duplicate error.
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        assert_value_match(
            EXPECTED_ADDED_RULES_YAML,
            &test_route_rules(&state),
        );

        let net_conf: NetConf =
            serde_yaml::from_str(REMOVE_ROUTE_RULE_YML).unwrap();
        net_conf.apply().unwrap();
        // Apply twice to test whether crate ignores not found error.
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        assert!(test_route_rules(&state).is_empty());
    })
}

fn test_route_rules(state: &NetState) -> Vec<RouteRule> {
    let mut rules: Vec<RouteRule> = state
        .rules
        .iter()
        .filter(|rule| {
            rule.priority
                .is_some_and(|prio| TEST_RULE_PRIORITIES.contains(&prio))
        })
        .cloned()
        .collect();
    rules.sort_unstable_by_key(|rule| rule.priority);
    rules
}

#[test]
fn test_remove_route_rules_preserves_unselected_records() {
    use rtnetlink::packet_route::{
        AddressFamily,
        rule::{RuleAction, RuleAttribute, RuleMessage},
    };

    with_route_rule_test_iface(|| {
        let _cleanup = RuleRemovalTestCleanup;
        let mut rules = Vec::new();
        for family in [AddressFamily::Inet, AddressFamily::Inet6] {
            for priority in [1000, 0] {
                let mut rule = RuleMessage::default();
                rule.header.family = family;
                rule.header.action = RuleAction::ToTable;
                rule.attributes = vec![
                    RuleAttribute::Table(500),
                    RuleAttribute::Priority(priority),
                    RuleAttribute::FwMark(1),
                    RuleAttribute::FwMask(255),
                ];
                rules.push(rule.clone());
                rules.push(rule.clone());
                rule.attributes[2] = RuleAttribute::FwMark(0);
                rules.push(rule.clone());
                rule.attributes = vec![
                    RuleAttribute::Table(501),
                    RuleAttribute::Priority(priority),
                    RuleAttribute::TunId(0x1_0000_0001),
                ];
                rules.push(rule);
            }
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap()
            .block_on(async {
                add_rule_removal_test_rules(rules).await;
                let before = kernel_rules_json();
                let expected: Vec<_> = before
                    .iter()
                    .filter(|rule| {
                        kernel_rule_number(rule, "table") != Some(500)
                            || kernel_rule_number(rule, "fwmark") != Some(0)
                    })
                    .cloned()
                    .collect();
                assert_eq!(before.len() - expected.len(), 4);
                assert_eq!(
                    expected
                        .iter()
                        .filter(|rule| {
                            kernel_rule_number(rule, "fwmark") == Some(1)
                        })
                        .count(),
                    8
                );
                assert_eq!(
                    expected
                        .iter()
                        .filter(|rule| {
                            kernel_rule_number(rule, "tun_id")
                                == Some(0x1_0000_0001)
                        })
                        .count(),
                    4
                );
                for _ in 0..2 {
                    crate::remove_route_rules(|rule| {
                        if rule.table == Some(501) {
                            assert_eq!(rule.tun_id, Some(0x1_0000_0001));
                        }
                        let remove = rule.table == Some(500)
                            && rule.fw_mark.unwrap_or(0) == 0;
                        if remove {
                            assert!(matches!(rule.priority, Some(0 | 1000)));
                            assert_eq!(rule.fw_mask, Some(255));
                        }
                        remove
                    })
                    .await
                    .unwrap();
                    assert_eq!(kernel_rules_json(), expected);
                }
                crate::remove_route_rules(|rule| rule.table == Some(500))
                    .await
                    .unwrap();
                assert_eq!(
                    kernel_rules_json(),
                    expected
                        .into_iter()
                        .filter(|rule| {
                            kernel_rule_number(rule, "table") != Some(500)
                        })
                        .collect::<Vec<_>>()
                );
            });
    });
}

async fn add_rule_removal_test_rules(
    rules: Vec<rtnetlink::packet_route::rule::RuleMessage>,
) {
    use futures::{
        StreamExt,
        future::{Either, select},
    };
    use rtnetlink::{
        packet_core::{
            NLM_F_ACK, NLM_F_CREATE, NLM_F_REQUEST, NetlinkMessage,
            NetlinkPayload,
        },
        packet_route::RouteNetlinkMessage,
    };

    let (mut connection, mut handle, _) = rtnetlink::new_connection().unwrap();
    connection.set_forward_ack(true);
    let add = async {
        for rule in rules {
            let mut request =
                NetlinkMessage::from(RouteNetlinkMessage::NewRule(rule));
            request.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE;
            let mut responses = handle.request(request).unwrap();
            let mut acknowledged = false;
            while let Some(response) = responses.next().await {
                let NetlinkPayload::Error(error) = response.payload else {
                    panic!("unexpected rule addition response {response:?}");
                };
                assert_eq!(error.code, None, "{error}");
                acknowledged = true;
            }
            assert!(acknowledged);
        }
    };
    assert!(matches!(
        select(Box::pin(add), connection).await,
        Either::Left(_)
    ));
}

fn kernel_rules_json() -> Vec<serde_json::Value> {
    let mut rules = Vec::new();
    for family in ["-4", "-6"] {
        let output = std::process::Command::new("ip")
            .args([family, "-details", "-json", "rule"])
            .output()
            .unwrap();
        assert!(output.status.success());
        rules.extend(
            serde_json::from_slice::<Vec<serde_json::Value>>(&output.stdout)
                .unwrap(),
        );
    }
    rules
}

fn kernel_rule_number(rule: &serde_json::Value, key: &str) -> Option<u64> {
    match &rule[key] {
        serde_json::Value::Number(number) => number.as_u64(),
        serde_json::Value::String(number) => {
            if let Some(hex) = number.strip_prefix("0x") {
                u64::from_str_radix(hex, 16).ok()
            } else {
                number.parse().ok()
            }
        }
        _ => None,
    }
}

struct RuleRemovalTestCleanup;

impl Drop for RuleRemovalTestCleanup {
    fn drop(&mut self) {
        for family in ["-4", "-6"] {
            for table in ["500", "501"] {
                let _ = std::process::Command::new("ip")
                    .args([family, "rule", "flush", "table", table])
                    .status();
            }
        }
    }
}

fn with_route_rule_test_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("rule");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
