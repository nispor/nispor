use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "tap1";

const EXPECTED_IFACE_NAME: &str = r#"---
- name: tap1
  iface_type: tun
  state: down
  mtu: 1500
  flags:
    - broadcast
    - multicast
    - up
  mac_address: "00:23:45:67:89:1c"
  tun:
    mode: tap
    owner: 1001
    group: 0
    pi: false
    vnet_hdr: true
    multi_queue: true
    persist: true
    num_queues: 0
    num_disabled_queues: 0"#;

#[test]
#[ignore] // Travis CI does not support TUN/TAP IFLA_INFO_DATA yet
fn test_get_tap_iface_yaml() {
    with_tap_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_NAME];
            assert_eq!(iface.iface_type, nispor::IfaceType::Tun);
            assert_eq!(
                serde_yaml::to_string(&vec![iface]).unwrap(),
                EXPECTED_IFACE_NAME
            );
        }
    });
}

fn with_tap_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("tap");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
