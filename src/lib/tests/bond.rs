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

const IFACE_NAME: &str = "bond99";
const PORT1_NAME: &str = "eth1";
const PORT2_NAME: &str = "eth2";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: bond99
  iface_type: bond
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - controller
    - multicast
    - running
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891c"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1c"
  bond:
    subordinates:
      - eth1
      - eth2
    mode: balance-rr
    miimon: 0
    updelay: 0
    downdelay: 0
    use_carrier: true
    arp_interval: 0
    arp_all_targets: any
    arp_validate: none
    resend_igmp: 1
    all_subordinates_active: dropped
    packets_per_subordinate: 1
- name: eth1
  iface_type: veth
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - subordinate
    - up
  mac_address: "00:23:45:67:89:1c"
  controller: bond99
  controller_type: bond
  bond_subordinate:
    subordinate_state: active
    mii_status: link_up
    link_failure_count: 0
    perm_hwaddr: "00:23:45:67:89:1a"
    queue_id: 0
  veth:
    peer: eth1.ep
- name: eth2
  iface_type: veth
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - subordinate
    - up
  mac_address: "00:23:45:67:89:1c"
  controller: bond99
  controller_type: bond
  bond_subordinate:
    subordinate_state: active
    mii_status: link_up
    link_failure_count: 0
    perm_hwaddr: "00:23:45:67:89:1b"
    queue_id: 0
  veth:
    peer: eth2.ep"#;

#[test]
fn test_get_iface_bond_yaml() {
    with_bond_iface(|| {
        let mut state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get_mut(IFACE_NAME).unwrap();
        // The peer_notif_delay is supported by kernel 5.3 and not
        // supported by Travis CI Ubuntu 18.04 kernel 4.15.
        if let Some(ref mut bond_info) = iface.bond {
            bond_info.peer_notif_delay = None;
        }
        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_eq!(&iface.iface_type, &nispor::IfaceType::Bond);
        assert_eq!(
            serde_yaml::to_string(&vec![iface, port1, port2])
                .unwrap()
                .trim(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_bond_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("bond");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
