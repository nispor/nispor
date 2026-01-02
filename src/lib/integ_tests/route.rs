// SPDX-License-Identifier: Apache-2.0

use super::utils::assert_value_match;
use crate::{NetConf, NetState, RouteProtocol};

const TEST_ROUTE_DST_V4: &str = "198.51.100.0/24";
const TEST_ROUTE_DST_V6: &str = "2001:db8:e::/64";

const EXPECTED_MULTIPATH_YAML_OUTPUT: &str = r#"---
- address-family: ipv6
  tos: 0
  table: 254
  protocol: static
  scope: universe
  route-type: unicast
  flags: []
  dst: "2001:db8:e::/64"
  cache-clntref: 0
  cache-last-use: 0
  cache-expires: 0
  cache-error: 0
  cache-used: 0
  cache-id: 0
  cache-ts: 0
  cache-ts-age: 0
  metric: 1024
  preference: medium
  multipath:
    - via: "2001:db8:f::254"
      iface: eth1
      weight: 1
      flags:
        - on-link
    - via: "2001:db8:f::253"
      iface: eth1
      weight: 256
      flags:
        - on-link
- address-family: ipv4
  tos: 0
  table: 254
  protocol: static
  scope: universe
  route-type: unicast
  flags: []
  dst: 198.51.100.0/24
  multipath:
    - via: 192.0.2.254
      iface: eth1
      weight: 1
      flags:
        - on-link
    - via: 192.0.2.253
      iface: eth1
      weight: 256
      flags:
        - on-link"#;

const EXPECTED_YAML_OUTPUT: &str = r#"---
- address-family: ipv4
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route-type: unicast
  flags: []
  oif: veth1
  gateway: 192.0.2.3
  metric: 500
- address-family: ipv4
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route-type: unicast
  flags: []
  dst: 198.51.100.0/24
  oif: veth1
  gateway: 192.0.2.2
  metric: 501
- address-family: ipv6
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route-type: unicast
  flags: []
  oif: veth1
  gateway: "2001:db8:a::3"
  cache-clntref: 0
  cache-last-use: 0
  cache-expires: 0
  cache-error: 0
  cache-used: 0
  cache-id: 0
  cache-ts: 0
  cache-ts-age: 0
  metric: 502
  preference: medium
- address-family: ipv6
  tos: 0
  table: 254
  protocol: dhcp
  scope: universe
  route-type: unicast
  flags: []
  dst: "2001:db8:e::/64"
  oif: veth1
  gateway: "2001:db8:a::2"
  cache-clntref: 0
  cache-last-use: 0
  cache-expires: 0
  cache-error: 0
  cache-used: 0
  cache-id: 0
  cache-ts: 0
  cache-ts-age: 0
  metric: 503
  preference: medium"#;

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
        let mut current_routes = Vec::new();
        for route in state.routes {
            if RouteProtocol::Dhcp == route.protocol
                && route.oif.as_deref() == Some("veth1")
            {
                current_routes.push(route)
            }
        }
        current_routes.sort_unstable_by_key(|r| r.metric);
        assert_value_match(EXPECTED_YAML_OUTPUT, &current_routes);

        let net_conf: NetConf = serde_yaml::from_str(REMOVE_ROUTE_YML).unwrap();
        net_conf.apply().unwrap();
        // Apply twice to test whether crate ignore the not found error.
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let mut current_routes = Vec::new();
        for route in state.routes {
            if RouteProtocol::Dhcp == route.protocol
                && route.oif.as_deref() == Some("veth1")
            {
                current_routes.push(route)
            }
        }
        assert!(current_routes.is_empty());
    })
}

const VETH_STATIC_IP_CONF: &str = r#"---
interfaces:
  - name: veth1
    type: veth
    state: up
    veth:
      peer: veth1.ep
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix-len: 24
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix-len: 64
  - name: veth1.ep
    type: veth
    state: up"#;

const VETH_ABSENT_CONF: &str = r#"---
interfaces:
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
        let mut current_routes = Vec::new();
        for route in state.routes {
            if Some(TEST_ROUTE_DST_V4.into()) == route.dst
                || Some(TEST_ROUTE_DST_V6.into()) == route.dst
            {
                current_routes.push(route)
            }
        }
        assert_value_match(EXPECTED_MULTIPATH_YAML_OUTPUT, &current_routes);
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

const TEST_ECMP_ROUTES: &str = r#"
- dst: 2001:db8:e::/64
  metric: 502
  protocol: dhcp
  table: 254
  multipath:
    - via: 2001:db8:a::3
      weight: 2
      iface: veth1
      flags:
        - on-link
    - via: 2001:db8:a::2
      weight: 1
      iface: veth1
      flags:
        - on-link
- dst: 198.51.100.0/24
  table: 254
  metric: 503
  protocol: dhcp
  multipath:
    - via: 192.0.2.254
      weight: 1
      iface: veth1
      flags:
        - on-link
    - via: 192.0.2.253
      weight: 2
      iface: veth1
      flags:
        - on-link
"#;

#[test]
fn test_add_and_remove_ecmp_route() {
    with_veth_static_ip(|| {
        let state_str = format!("routes:\n{TEST_ECMP_ROUTES}");
        let net_conf: NetConf = serde_yaml::from_str(&state_str).unwrap();
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let mut current_routes = Vec::new();
        for route in state.routes {
            if RouteProtocol::Dhcp == route.protocol && route.oif.is_none() {
                current_routes.push(route)
            }
        }

        current_routes.sort_unstable_by_key(|r| r.metric);
        assert_value_match(TEST_ECMP_ROUTES, &current_routes);
    })
}
