use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "eth1.101";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: eth1.101
  iface_type: vlan
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
  vlan:
    vlan_id: 101
    protocol: 802.1q
    base_iface: eth1
    is_reorder_hdr: true
    is_gvrp: false
    is_loose_binding: false
    is_mvrp: false
    is_bridge_binding: false"#;

#[test]
fn test_get_vlan_iface_yaml() {
    with_vlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, nispor::IfaceType::Vlan);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_vlan_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("vlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
