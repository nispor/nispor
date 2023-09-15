// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::{NetConf, NetState};

use super::utils::assert_value_match;

const IFACE_NAME: &str = "br0";
const PORT1_NAME: &str = "eth1";
const PORT2_NAME: &str = "eth2";

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
    - eth1
    - eth2
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
  vlan_stats_per_host: false
  stp_state: disabled
  hello_timer: 0
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
stp_path_cost: 2
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
forward_delay_timer: 0
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
stp_path_cost: 2
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
forward_delay_timer: 0
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

#[test]
fn test_get_br_iface_yaml() {
    with_br_iface(|| {
        let mut state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get_mut(IFACE_NAME).unwrap();
        if let Some(ref mut bridge_info) = iface.bridge {
            bridge_info.gc_timer = None;
            // Below value is not supported by RHEL 8 and Ubuntu CI
            bridge_info.multi_bool_opt = None;
            // Below value is different between CI and RHEL/CentOS 8
            // https://blog.grisge.info/posts/br_on_250hz_kernel/
            bridge_info.multicast_startup_query_interval = None;
        }
        let port1 = state.ifaces.get_mut(PORT1_NAME).unwrap();
        if let Some(ref mut port_info) = port1.bridge_port {
            port_info.forward_delay_timer = 0;
            // Below values are not supported by Github CI Ubuntu 20.04
            port_info.mrp_in_open = None;
        }
        let port2 = state.ifaces.get_mut(PORT2_NAME).unwrap();
        if let Some(ref mut port_info) = port2.bridge_port {
            port_info.forward_delay_timer = 0;
            port_info.mrp_in_open = None;
        }

        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Bridge);

        assert_value_match(EXPECTED_BRIDGE_INFO, iface);
        assert_value_match(EXPECTED_PORT1_BRIDGE_INFO, &port1.bridge_port);
        assert_value_match(EXPECTED_PORT2_BRIDGE_INFO, &port2.bridge_port);
    });
}

fn with_br_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("br");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

const BRIDGE_CREATE_YML: &str = r#"---
ifaces:
  - name: br0
    type: bridge"#;

const BRIDGE_DELETE_YML: &str = r#"---
ifaces:
  - name: br0
    type: bridge
    state: absent"#;

#[test]
fn test_create_delete_bridge() {
    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_CREATE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(&iface.iface_type, &crate::IfaceType::Bridge);

    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
}
