use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "mac0";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: mac0
  iface_type: mac_vlan
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
      - address: "fe80::223:45ff:fe67:891f"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1f"
  mac_vlan:
    base_iface: eth1
    mode: source
    flags: 0
    allowed_mac_addresses:
      - "00:23:45:67:89:1d"
      - "00:23:45:67:89:1c""#;

#[test]
fn test_get_macvlan_iface_yaml() {
    with_macvlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, nispor::IfaceType::MacVlan);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_macvlan_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("macvlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
