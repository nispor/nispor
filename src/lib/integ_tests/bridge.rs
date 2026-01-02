// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{BridgeStpState, NetConf, NetState};

const IFACE_NAME: &str = "br0";
const PORT1_NAME: &str = "dummy1";
const PORT2_NAME: &str = "dummy2";

// On Archlinux where HZ == 300, these properties will be rounded up by
// `jiffies_to_clock_t()` of kernel:
//  * ageing-time
//  * hello-time
//  * forward-delay
//  * max-age
//  * multicast-last-member-interval
//  * multicast-membership-interval
//  * multicast-querier-interval
//  * multicast-query-interval
//  * multicast-query-response-interval
//  Hence we skip those from testing
//  Ubuntu is 250 HZ, which hold a subset of above list.

const EXPECTED_BRIDGE_INFO: &str = r#"---
name: br0
type: linux-bridge
bridge:
  ports:
    - dummy1
    - dummy2
  bridge-id: 8000.00234567891c
  group-fwd-mask: 0
  root-id: 8000.00234567891c
  root-port: 0
  root-path-cost: 0
  topology-change: false
  topology-change-detected: false
  tcn-timer: 0
  topology-change-timer: 0
  group-addr: "01:80:c2:00:00:00"
  nf-call-iptables: false
  nf-call-ip6tables: false
  nf-call-arptables: false
  vlan-filtering: false
  vlan-protocol: 802.1q
  default-pvid: 1
  vlan-stats-enabled: false
  vlan-stats-per-port: false
  stp-state: disabled
  priority: 32768
  multicast-router: temp-query
  multicast-snooping: true
  multicast-query-use-ifaddr: false
  multicast-querier: false
  multicast-stats-enabled: false
  multicast-hash-elasticity: 16
  multicast-hash-max: 4096
  multicast-last-member-count: 2
  multicast-startup-query-count: 2
  multicast-igmp-version: 2
  multicast-mld-version: 1"#;

const EXPECTED_PORT1_BRIDGE_INFO: &str = r#"---
stp-state: forwarding
stp-priority: 60
stp-path-cost: 1000
hairpin-mode: true
bpdu-guard: true
root-block: true
multicast-fast-leave: true
learning: true
unicast-flood: true
proxy-arp: true
proxy-arp-wifi: true
designated-root: 8000.00234567891c
designated-bridge: 8000.00234567891c
designated-port: 61441
designated-cost: 0
port-id: "0xf001"
port-no: "0x1"
change-ack: false
config-pending: false
message-age-timer: 0
hold-timer: 0
multicast-router: perm
multicast-flood: true
multicast-to-unicast: true
vlan-tunnel: false
broadcast-flood: true
group-fwd-mask: 1
neigh-suppress: true
isolated: true
mrp-ring-open: false
mcast-eht-hosts-limit: 512
mcast-eht-hosts-cnt: 0
vlans:
  - vid: 1
    is-pvid: true
    is-egress-untagged: true"#;

const EXPECTED_PORT2_BRIDGE_INFO: &str = r#"---
stp-state: forwarding
stp-priority: 50
stp-path-cost: 1001
hairpin-mode: true
bpdu-guard: true
root-block: true
multicast-fast-leave: true
learning: true
unicast-flood: true
proxy-arp: true
proxy-arp-wifi: true
designated-root: 8000.00234567891c
designated-bridge: 8000.00234567891c
designated-port: 51202
designated-cost: 0
port-id: "0xc802"
port-no: "0x2"
change-ack: false
config-pending: false
message-age-timer: 0
hold-timer: 0
multicast-router: perm
multicast-flood: true
multicast-to-unicast: true
vlan-tunnel: false
broadcast-flood: true
group-fwd-mask: 1
neigh-suppress: true
isolated: true
mrp-ring-open: false
mcast-eht-hosts-limit: 512
mcast-eht-hosts-cnt: 0
vlans:
  - vid: 1
    is-pvid: true
    is-egress-untagged: true"#;

const BRIDGE_CREATE_YML: &str = r#"---
interfaces:
  - name: br0
    type: linux-bridge
    mac-address: 00:23:45:67:89:1c
    bridge:
      stp-state: disabled
  - name: dummy1
    type: dummy
    state: up
    controller: br0
    bridge-port:
      flush: false
      stp-priority: 60
      stp-path-cost: 1000
      hairpin-mode: true
      bpdu-guard: true
      root-block: true
      multicast-fast-leave: true
      learning: true
      unicast-flood: true
      proxy-arp: true
      proxy-arp-wifi: true
      multicast-router: perm
      multicast-flood: true
      multicast-to-unicast: true
      vlan-tunnel: false
      broadcast-flood: true
      group-fwd-mask: 1
      neigh-suppress: true
      isolated: true
      mac-authentication-bypass: true
      backup-port: 0
      locked: true
      neigh-vlan-suppress: false
      backup-nexthop-id: 0
  - name: dummy2
    type: dummy
    state: up
    controller: br0
    bridge-port:
      flush: false
      stp-priority: 50
      stp-path-cost: 1001
      hairpin-mode: true
      bpdu-guard: true
      root-block: true
      multicast-fast-leave: true
      learning: true
      unicast-flood: true
      proxy-arp: true
      proxy-arp-wifi: true
      multicast-router: perm
      multicast-flood: true
      multicast-to-unicast: true
      vlan-tunnel: false
      broadcast-flood: true
      group-fwd-mask: 1
      neigh-suppress: true
      isolated: true
      mac-authentication-bypass: true
      backup-port: 0
      locked: true
      neigh-vlan-suppress: false
      backup-nexthop-id: 0
    "#;

const BRIDGE_DELETE_YML: &str = r#"---
interfaces:
  - name: br0
    type: linux-bridge
    state: absent
  - name: dummy1
    type: dummy
    state: absent
  - name: dummy2
    type: dummy
    state: absent"#;

fn with_br_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    assert!(result.is_ok())
}

#[test]
fn test_create_delete_bridge() {
    with_br_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Bridge);

        assert_value_match(EXPECTED_BRIDGE_INFO, iface);
        assert_value_match(EXPECTED_PORT1_BRIDGE_INFO, &port1.bridge_port);
        assert_value_match(EXPECTED_PORT2_BRIDGE_INFO, &port2.bridge_port);
    });
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
}

#[test]
fn test_bridge_change_stp_state() {
    with_br_iface(|| {
        let net_conf: NetConf = serde_yaml::from_str(
            r#"---
                interfaces:
                - name: br0
                  type: linux-bridge
                  bridge:
                    stp-state: kernel-stp
                "#,
        )
        .unwrap();
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Bridge);

        assert_eq!(
            iface.bridge.as_ref().and_then(|b| b.stp_state.as_ref()),
            Some(&BridgeStpState::KernelStp)
        );
    });
}
