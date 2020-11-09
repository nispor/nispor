use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "vrf0";

const EXPECTED_IFACE_NAME: &str = r#"---
- name: vrf0
  iface_type: vrf
  state: up
  mtu: 65536
  flags:
    - lower_up
    - controller
    - no_arp
    - running
    - up
  mac_address: "00:23:45:67:89:1c"
  vrf:
    table_id: 10
    subordinates:
      - eth1
      - eth2"#;

#[test]
#[ignore] // Travis CI does not support VRF yet
fn test_get_vrf_iface_yaml() {
    with_vrf_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_NAME];
            assert_eq!(iface.iface_type, nispor::IfaceType::Vrf);
            assert_eq!(
                serde_yaml::to_string(&vec![iface]).unwrap(),
                EXPECTED_IFACE_NAME
            );
        }
    });
}

fn with_vrf_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("vrf");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
