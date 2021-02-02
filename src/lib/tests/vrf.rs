use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "vrf0";

const EXPECTED_IFACE_STATE: &str = r#"---
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
  vrf:
    table_id: 10
    subordinates:
      - eth1
      - eth2"#;

#[test]
#[ignore] // Github Action does not have VRF kernel module
fn test_get_vrf_iface_yaml() {
    with_vrf_iface(|| {
        let state = NetState::retrieve().unwrap();
        let mut iface = state.ifaces[IFACE_NAME].clone();
        // RHEL/CentOS 8 and Ubuntu 20.04 does not support changing mac
        // address of VRF interface
        iface.mac_address = "".into();
        assert_eq!(iface.iface_type, nispor::IfaceType::Vrf);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE
        );
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
