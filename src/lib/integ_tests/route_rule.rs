// SPDX-License-Identifier: Apache-2.0

use crate::NetState;

use std::panic;

use super::utils::assert_value_match;

const TEST_TABLE_ID: u32 = 100;

const EXPECTED_YAML_OUTPUT: &str = r#"---
- action: blackhole
  address_family: ipv6
  flags: 0
  tos: 0
  table: 100
  dst: 2001:db8:f::252/128
  src: 2001:db8:f::255/128
  iif: eth1
  oif: eth2
  priority: 998
- action: table
  address_family: ipv6
  flags: 0
  tos: 16
  table: 100
  dst: "2001:db8:f::253/128"
  src: "2001:db8:f::254/128"
  iif: eth1
  oif: eth2
  priority: 999
- action: unreachable
  address_family: ipv4
  flags: 0
  tos: 0
  dst: 192.0.2.2/32
  src: 192.0.2.1/32
  iif: eth1
  oif: eth2
  priority: 998
- action: table
  address_family: ipv4
  flags: 0
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
