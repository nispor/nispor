use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "br0";
const PORT1_NAME: &str = "eth1";
const PORT2_NAME: &str = "eth2";

const EXPECTED_IFACE_STATE: &str = r#"---
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
        is_pvid: false
        is_egress_untagged: true
      - vid: 10
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
      - vid_range:
          - 2
          - 4094
        is_pvid: false
        is_egress_untagged: false
  veth:
    peer: eth2.ep"#;

#[test]
fn test_get_br_iface_yaml() {
    with_br_with_vlan_filter_iface(|| {
        let mut state = NetState::retrieve().unwrap();
        let port1 = state.ifaces.get_mut(PORT1_NAME).unwrap();
        if let Some(ref mut port_info) = port1.bridge_port {
            port_info.forward_delay_timer = 0;
        }
        let port2 = state.ifaces.get_mut(PORT2_NAME).unwrap();
        if let Some(ref mut port_info) = port2.bridge_port {
            port_info.forward_delay_timer = 0;
        }
        let iface = state.ifaces.get(IFACE_NAME).unwrap();
        if let Some(bridge_info) = &iface.bridge {
            assert_eq!(bridge_info.vlan_filtering, Some(true))
        }

        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_eq!(
            serde_yaml::to_string(&vec![port1, port2]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_br_with_vlan_filter_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("brv");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
