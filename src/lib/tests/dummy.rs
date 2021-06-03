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

const IFACE_NAME: &str = "dummy1";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: dummy1
  iface_type: dummy
  state: unknown
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - no_arp
    - running
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891a"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1a""#;

const EXPECTED_ETHTOOL_FEATURE: &str = r#"---
fixed:
  esp-hw-offload: false
  esp-tx-csum-hw-offload: false
  fcoe-mtu: false
  hw-tc-offload: false
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
  rx-udp_tunnel-port-offload: false
  rx-vlan-filter: false
  rx-vlan-hw-parse: false
  rx-vlan-stag-filter: false
  rx-vlan-stag-hw-parse: false
  tls-hw-record: false
  tls-hw-rx-offload: false
  tls-hw-tx-offload: false
  tx-checksum-fcoe-crc: false
  tx-checksum-ipv4: false
  tx-checksum-ipv6: false
  tx-checksum-sctp: false
  tx-esp-segmentation: false
  tx-fcoe-segmentation: false
  tx-gso-list: false
  tx-gso-partial: false
  tx-gso-robust: false
  tx-lockless: true
  tx-sctp-segmentation: false
  tx-tunnel-remcsum-segmentation: false
  tx-udp-segmentation: false
  tx-vlan-hw-insert: false
  tx-vlan-stag-hw-insert: false
  vlan-challenged: false
changeable:
  highdma: true
  rx-gro: true
  rx-gro-list: false
  tx-checksum-ip-generic: true
  tx-generic-segmentation: true
  tx-gre-csum-segmentation: true
  tx-gre-segmentation: true
  tx-ipxip4-segmentation: true
  tx-ipxip6-segmentation: true
  tx-nocache-copy: false
  tx-scatter-gather-fraglist: true
  tx-tcp-ecn-segmentation: true
  tx-tcp-mangleid-segmentation: true
  tx-tcp-segmentation: true
  tx-tcp6-segmentation: true
  tx-udp_tnl-csum-segmentation: true
  tx-udp_tnl-segmentation: true"#;

#[test]
fn test_get_iface_dummy_yaml() {
    with_dummy_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Dummy);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE
        );
    });
}

#[test]
#[ignore] // CI does not have ethtool_netlink kernel module yet
fn test_get_iface_dummy_ethtool_feature() {
    with_dummy_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(
            serde_yaml::to_string(&iface.ethtool.as_ref().unwrap().features)
                .unwrap(),
            EXPECTED_ETHTOOL_FEATURE
        );
    });
}

fn with_dummy_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("dummy");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
