use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "br0";
const PORT1_NAME: &str = "eth1";
const PORT2_NAME: &str = "eth2";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: br0
  iface_type: bridge
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
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
  bridge:
    ports:
      - eth1
      - eth2
    ageing_time: 30000
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
    stp_state: disabled
    hello_time: 200
    hello_timer: 0
    forward_delay: 1500
    max_age: 2000
    priority: 32768
    multicast_router: temp_query
    multicast_snooping: true
    multicast_query_use_ifaddr: false
    multicast_querier: false
    multicast_stats_enabled: false
    multicast_last_member_count: 2
    multicast_last_member_interval: 100
    multicast_startup_query_count: 2
    multicast_membership_interval: 26000
    multicast_querier_interval: 25500
    multicast_query_interval: 12500
    multicast_query_response_interval: 1000
    multicast_igmp_version: 2
    multicast_mld_version: 1
- name: eth1
  iface_type: veth
  state: up
  mtu: 1500
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891a"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1a"
  controller: br0
  controller_type: bridge
  bridge_port:
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
    vlans:
      - vid: 1
        is_pvid: true
        is_egress_untagged: true
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
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891b"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1b"
  controller: br0
  controller_type: bridge
  bridge_port:
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
    vlans:
      - vid: 1
        is_pvid: true
        is_egress_untagged: true
  veth:
    peer: eth2.ep"#;

#[test]
fn test_get_br_iface_yaml() {
    with_br_iface(|| {
        let mut state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get_mut(IFACE_NAME).unwrap();
        if let Some(ref mut bridge_info) = iface.bridge {
            bridge_info.gc_timer = None;
            // Below are not support by Travis CI kernel
            bridge_info.vlan_stats_per_host = None;
            bridge_info.multi_bool_opt = None;
            // Below are diffent value between Travis CI and RHEL/CentOS 8
            bridge_info.multicast_hash_elasticity = None;
            bridge_info.multicast_hash_max = None;
            bridge_info.multicast_startup_query_interval = None;
        }
        let port1 = state.ifaces.get_mut(PORT1_NAME).unwrap();
        if let Some(ref mut port_info) = port1.bridge_port {
            port_info.forward_delay_timer = 0;
        }
        let port2 = state.ifaces.get_mut(PORT2_NAME).unwrap();
        if let Some(ref mut port_info) = port2.bridge_port {
            port_info.forward_delay_timer = 0;
        }

        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_eq!(iface.iface_type, nispor::IfaceType::Bridge);
        assert_eq!(
            serde_yaml::to_string(&vec![iface, port1, port2])
                .unwrap()
                .trim(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_br_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("br");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
