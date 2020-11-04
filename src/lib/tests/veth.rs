use nispor::NetState;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "veth1";

const EXPECTED_IFACE_NAME: &str = r#"---
- name: veth1
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
  veth:
    peer: veth1.ep"#;

#[test]
fn test_get_veth_iface_yaml() {
    with_veth_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_NAME];
            let iface_type = &iface.iface_type;
            assert_eq!(iface_type, &nispor::IfaceType::Veth);
            assert_eq!(
                serde_yaml::to_string(&vec![iface]).unwrap(),
                EXPECTED_IFACE_NAME
            );
        }
    });
}

fn with_veth_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("veth");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
