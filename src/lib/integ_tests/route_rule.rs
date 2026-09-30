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
