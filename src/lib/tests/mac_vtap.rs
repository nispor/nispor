use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "macvtap0";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: macvtap0
  iface_type: mac_vtap
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
  mac_vtap:
    base_iface: eth1
    mode: source
    flags: 0
    allowed_mac_addresses:
      - "00:23:45:67:89:1c"
      - "00:23:45:67:89:1b""#;

#[test]
fn test_get_macvtap_iface_yaml() {
    with_macvtap_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, nispor::IfaceType::MacVtap);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_macvtap_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("macvtap");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
