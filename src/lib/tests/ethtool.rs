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

const IFACE_NAME0: &str = "sim0";
const IFACE_NAME1: &str = "sim1";

const EXPECTED_IFACE_STATE0: &str = r#"---
- name: sim0
  iface_type: ethernet
  state: down
  mtu: 1500
  flags:
    - broadcast
    - no_arp
  mac_address: "00:23:45:67:89:20"
  ethtool:
    pause:
      rx: true
      tx: true
      auto_negotiate: false
  sriov:
    vfs: []"#;

const EXPECTED_IFACE_STATE1: &str = r#"---
- name: sim1
  iface_type: ethernet
  state: down
  mtu: 1500
  flags:
    - broadcast
    - no_arp
  mac_address: "00:23:45:67:89:21"
  ethtool:
    pause:
      rx: true
      tx: true
      auto_negotiate: false
  sriov:
    vfs: []"#;

#[test]
#[ignore] // CI does not have netdevsim kernel module yet
fn test_get_ethtool_pause_yaml() {
    with_ethtool_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME0];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Ethernet);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE0
        );
        let iface = &state.ifaces[IFACE_NAME1];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Ethernet);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE1
        );
    });
}

fn with_ethtool_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("ethtool");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
