// SPDX-License-Identifier: Apache-2.0

use crate::{NetConf, NetState, RouteProtocol};

use super::utils::assert_value_match;

const TEST_ROUTE_DST_V4: &str = "198.51.100.0/24";
const TEST_ROUTE_DST_V6: &str = "2001:db8:e::/64";

const EXPECTED_MULTIPATH_YAML_OUTPUT: &str = r#"---
- address_family: ipv6
  tos: 0
  table: 254
  protocol: static
  scope: universe
  route_type: unicast
  flags: 0
  dst: "2001:db8:e::/64"
  cache_clntref: 0
  cache_last_use: 0
  cache_expires: 0
  cache_error: 0
  cache_used: 0
  cache_id: 0
  cache_ts: 0
  cache_ts_age: 0
  metric: 1024
  perf: 0
  multipath:
    - via: "2001:db8:f::254"
      iface: eth1
      weight: 1
      flags:
        - on_link
    - via: "2001:db8:f::253"
      iface: eth1
      weight: 256
      flags:
        - on_link
- address_family: ipv4
  tos: 0
  table: 254
  protocol: static
  scope: universe
  route_type: unicast
  flags: 0
  dst: 198.51.100.0/24
  multipath:
    - via: 192.0.2.254
      iface: eth1
      weight: 1
      flags:
        - on_link
    - via: 192.0.2.253
      iface: eth1
      weight: 256
      flags:
        - on_link"#;

const EXPECTED_YAML_OUTPUT: &str = r#"---
- address_family: ipv4
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route_type: unicast
  flags: 0
  oif: veth1
  gateway: 192.0.2.3
  metric: 500
- address_family: ipv4
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route_type: unicast
  flags: 0
  dst: 198.51.100.0/24
  oif: veth1
  gateway: 192.0.2.2
  metric: 501
- address_family: ipv6
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route_type: unicast
  flags: 0
  oif: veth1
  gateway: "2001:db8:a::3"
  cache_clntref: 0
  cache_last_use: 0
  cache_expires: 0
  cache_error: 0
  cache_used: 0
  cache_id: 0
  cache_ts: 0
  cache_ts_age: 0
  metric: 502
  perf: 0
- address_family: ipv6
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route_type: unicast
  flags: 0
  dst: "2001:db8:e::/64"
  oif: veth1
  gateway: "2001:db8:a::2"
  cache_clntref: 0
  cache_last_use: 0
  cache_expires: 0
  cache_error: 0
  cache_used: 0
  cache_id: 0
  cache_ts: 0
  cache_ts_age: 0
  metric: 503
  perf: 0"#;

const ADD_ROUTE_YML: &str = r#"---
routes:
- dst: 0.0.0.0/0
  oif: veth1
  via: 192.0.2.3
  metric: 500
  protocol: dhcp
  table: 254
- dst: 198.51.100.0/24
  oif: veth1
  via: 192.0.2.2
  metric: 501
  protocol: dhcp
  table: 254
- dst: ::/0
  oif: veth1
  via: 2001:db8:a::3
  metric: 502
  protocol: dhcp
  table: 254
- dst: 2001:db8:e::/64
  oif: veth1
  via: 2001:db8:a::2
  metric: 503
  protocol: dhcp
  table: 254"#;

const REMOVE_ROUTE_YML: &str = r#"---
routes:
- dst: 0.0.0.0/0
  oif: veth1
  via: 192.0.2.3
  metric: 500
  protocol: dhcp
  table: 254
  remove: true
- dst: 198.51.100.0/24
  oif: veth1
  via: 192.0.2.2
  metric: 501
  protocol: dhcp
  table: 254
  remove: true
- dst: ::/0
  oif: veth1
  via: 2001:db8:a::3
  metric: 502
  protocol: dhcp
  table: 254
  remove: true
- dst: 2001:db8:e::/64
  oif: veth1
  via: 2001:db8:a::2
  metric: 503
  protocol: dhcp
  table: 254
  remove: true"#;

#[test]
fn test_add_remove_route_yaml() {
    with_veth_static_ip(|| {
        let net_conf: NetConf = serde_yaml::from_str(ADD_ROUTE_YML).unwrap();
        net_conf.apply().unwrap();
        // Apply twice to test whether crate ignore duplicate error.
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let mut expected_routes = Vec::new();
        for route in state.routes {
            if RouteProtocol::Dhcp == route.protocol
                && route.oif.as_deref() == Some("veth1")
            {
                expected_routes.push(route)
            }
        }
        expected_routes.sort_unstable_by_key(|r| r.metric);
        assert_value_match(EXPECTED_YAML_OUTPUT, &expected_routes);

        let net_conf: NetConf = serde_yaml::from_str(REMOVE_ROUTE_YML).unwrap();
        net_conf.apply().unwrap();
        // Apply twice to test whether crate ignore the not found error.
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let mut expected_routes = Vec::new();
        for route in state.routes {
            if RouteProtocol::Dhcp == route.protocol
                && route.oif.as_deref() == Some("veth1")
            {
                expected_routes.push(route)
            }
        }
        assert!(expected_routes.is_empty());
    })
}

const VETH_STATIC_IP_CONF: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    veth:
      peer: veth1.ep
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix_len: 24
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix_len: 64"#;

const VETH_ABSENT_CONF: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    state: absent"#;

fn with_veth_static_ip<T>(test: T)
where
    T: FnOnce() + std::panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(VETH_STATIC_IP_CONF).unwrap();
    net_conf.apply().unwrap();
    let result = std::panic::catch_unwind(|| {
        test();
    });
    let net_conf: NetConf = serde_yaml::from_str(VETH_ABSENT_CONF).unwrap();
    net_conf.apply().unwrap();
    assert!(result.is_ok())
}

#[test]
fn test_get_route_yaml() {
    with_route_test_iface(|| {
        let state = NetState::retrieve().unwrap();
        let mut expected_routes = Vec::new();
        for route in state.routes {
            if Some(TEST_ROUTE_DST_V4.into()) == route.dst
                || Some(TEST_ROUTE_DST_V6.into()) == route.dst
            {
                expected_routes.push(route)
            }
        }
        assert_value_match(EXPECTED_MULTIPATH_YAML_OUTPUT, &expected_routes);
    });
}

fn with_route_test_iface<T>(test: T)
where
    T: FnOnce() + std::panic::UnwindSafe,
{
    super::utils::set_network_environment("route");

    let result = std::panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
