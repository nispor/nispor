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

use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const TEST_ROUTE_DST_V4: &str = "198.51.100.0/24";
const TEST_ROUTE_DST_V6: &str = "2001:db8:e::/64";

const EXPECTED_YAML_OUTPUT: &str = r#"---
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

#[test]
fn test_get_route_yaml() {
    with_route_test_iface(|| {
        let state = NetState::retrieve().unwrap();
        let mut expected_routes = Vec::new();
        for route in state.routes {
            if Some(TEST_ROUTE_DST_V4.into()) == route.dst {
                expected_routes.push(route)
            } else if Some(TEST_ROUTE_DST_V6.into()) == route.dst {
                expected_routes.push(route)
            }
        }
        assert_eq!(
            serde_yaml::to_string(&expected_routes).unwrap().trim(),
            EXPECTED_YAML_OUTPUT
        );
    });
}

fn with_route_test_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("route");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
