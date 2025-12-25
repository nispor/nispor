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
//  * ageing_time
//  * hello_time
//  * forward_delay
//  * max_age
//  * multicast_last_member_interval
//  * multicast_membership_interval
//  * multicast_querier_interval
//  * multicast_query_interval
//  * multicast_query_response_interval
//  Hence we skip those from testing
//  Ubuntu is 250 HZ, which hold a subset of above list.

const EXPECTED_BRIDGE_INFO: &str = r#"---
name: br0
iface_type: bridge
bridge:
  ports:
    - dummy1
    - dummy2
  bridge_id: 8000.00234567891c
  group_fwd_mask: 0
  root_id: 8000.00234567891c
  root_port: 0
  root_path_cost: 0
  topology_change: false
  topology_change_detected: false
  tcn_timer: 0
  topology_change_timer: 0
  group_addr: "01:80:c2:00:00:00"
  nf_call_iptables: false
  nf_call_ip6tables: false
  nf_call_arptables: false
  vlan_filtering: false
  vlan_protocol: 802.1q
  default_pvid: 1
  vlan_stats_enabled: false
  vlan_stats_per_port: false
  stp_state: disabled
  priority: 32768
  multicast_router: temp_query
  multicast_snooping: true
  multicast_query_use_ifaddr: false
  multicast_querier: false
  multicast_stats_enabled: false
  multicast_hash_elasticity: 16
  multicast_hash_max: 4096
  multicast_last_member_count: 2
  multicast_startup_query_count: 2
  multicast_igmp_version: 2
  multicast_mld_version: 1"#;

const EXPECTED_PORT1_BRIDGE_INFO: &str = r#"---
stp_state: forwarding
stp_priority: 32
stp_path_cost: 100
hairpin_mode: false
bpdu_guard: false
root_block: false
multicast_fast_leave: false
learning: true
unicast_flood: true
proxyarp: false
proxyarp_wifi: false
designated_root: 8000.00234567891c
designated_bridge: 8000.00234567891c
designated_port: 32769
designated_cost: 0
port_id: "0x8001"
port_no: "0x1"
change_ack: false
config_pending: false
message_age_timer: 0
hold_timer: 0
multicast_router: temp_query
multicast_flood: true
multicast_to_unicast: false
vlan_tunnel: false
broadcast_flood: true
group_fwd_mask: 0
neigh_suppress: false
isolated: false
mrp_ring_open: false
mcast_eht_hosts_limit: 512
mcast_eht_hosts_cnt: 0
vlans:
  - vid: 1
    is_pvid: true
    is_egress_untagged: true"#;

const EXPECTED_PORT2_BRIDGE_INFO: &str = r#"---
stp_state: forwarding
stp_priority: 32
stp_path_cost: 100
hairpin_mode: false
bpdu_guard: false
root_block: false
multicast_fast_leave: false
learning: true
unicast_flood: true
proxyarp: false
proxyarp_wifi: false
designated_root: 8000.00234567891c
designated_bridge: 8000.00234567891c
designated_port: 32770
designated_cost: 0
port_id: "0x8002"
port_no: "0x2"
change_ack: false
config_pending: false
message_age_timer: 0
hold_timer: 0
multicast_router: temp_query
multicast_flood: true
multicast_to_unicast: false
vlan_tunnel: false
broadcast_flood: true
group_fwd_mask: 0
neigh_suppress: false
isolated: false
mrp_ring_open: false
mcast_eht_hosts_limit: 512
mcast_eht_hosts_cnt: 0
vlans:
  - vid: 1
    is_pvid: true
    is_egress_untagged: true"#;

const BRIDGE_CREATE_YML: &str = r#"---
interfaces:
  - name: br0
    type: bridge
    mac-address: 00:23:45:67:89:1c
    bridge:
      stp-state: disabled
  - name: dummy1
    type: dummy
    state: up
    controller: br0
  - name: dummy2
    type: dummy
    state: up
    controller: br0
    "#;

const BRIDGE_DELETE_YML: &str = r#"---
interfaces:
  - name: br0
    type: bridge
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
                    stp_state: kernel_stp
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
