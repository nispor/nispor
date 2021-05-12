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

const EXPECTED_IFACE_STATE: &str = r#"---
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
    features:
      fixed:
        esp-hw-offload: true
        esp-tx-csum-hw-offload: true
        fcoe-mtu: false
        highdma: true
        l2-fwd-offload: false
        loopback: false
        macsec-hw-offload: false
        netns-local: false
        rx-all: false
        rx-checksum: false
        rx-fcs: false
        rx-gro-hw: false
        rx-hashing: false
        rx-lro: false
        rx-ntuple-filter: false
        rx-vlan-filter: false
        rx-vlan-hw-parse: false
        rx-vlan-stag-filter: false
        rx-vlan-stag-hw-parse: false
        tls-hw-record: false
        tls-hw-rx-offload: false
        tls-hw-tx-offload: false
        tx-checksum-fcoe-crc: false
        tx-checksum-ip-generic: true
        tx-checksum-ipv4: false
        tx-checksum-ipv6: false
        tx-checksum-sctp: false
        tx-esp-segmentation: true
        tx-fcoe-segmentation: false
        tx-gre-csum-segmentation: false
        tx-gre-segmentation: false
        tx-gso-list: false
        tx-gso-partial: false
        tx-gso-robust: false
        tx-ipxip4-segmentation: false
        tx-ipxip6-segmentation: false
        tx-lockless: false
        tx-scatter-gather-fraglist: true
        tx-sctp-segmentation: false
        tx-tcp-ecn-segmentation: false
        tx-tcp-mangleid-segmentation: false
        tx-tcp-segmentation: true
        tx-tcp6-segmentation: false
        tx-tunnel-remcsum-segmentation: false
        tx-udp-segmentation: false
        tx-udp_tnl-csum-segmentation: false
        tx-udp_tnl-segmentation: false
        tx-vlan-hw-insert: false
        tx-vlan-stag-hw-insert: false
        vlan-challenged: false
      changeable:
        hw-tc-offload: false
        rx-gro: true
        rx-gro-list: false
        rx-udp_tunnel-port-offload: true
        tx-generic-segmentation: true
        tx-nocache-copy: false
  sriov:
    vfs: []"#;

#[test]
#[ignore] // CI does not have netdevsim kernel module yet
fn test_get_ethtool_yaml() {
    with_ethtool_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME0];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Ethernet);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE
        );

        let state1 = EXPECTED_IFACE_STATE
            .replace(IFACE_NAME0, IFACE_NAME1)
            .replace("00:23:45:67:89:20", "00:23:45:67:89:21");

        let iface = &state.ifaces[IFACE_NAME1];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Ethernet);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            &state1
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
