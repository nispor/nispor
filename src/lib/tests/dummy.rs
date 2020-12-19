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

#[test]
fn test_get_iface_dummy_yaml() {
    with_dummy_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Dummy);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap(),
            EXPECTED_IFACE_STATE
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
