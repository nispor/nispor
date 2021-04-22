use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "sim0";

const EXPECTED_IFACE_STATE: &str = r#"---
- name: sim0
  iface_type: ethernet
  state: down
  mtu: 1500
  flags:
    - broadcast
    - no_arp
  mac_address: "00:23:45:67:89:20"
  ethtool:
    pause:
      rx: true
      tx: true
      auto_negotiate: false
  sriov:
    vfs: []"#;

#[test]
#[ignore] // CI does not have netdevsim kernel module yet
fn test_get_ethtool_pause_yaml() {
    with_ethtool_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Ethernet);
        assert_eq!(
            serde_yaml::to_string(&vec![iface]).unwrap().trim(),
            EXPECTED_IFACE_STATE
        );
    });
}

fn with_ethtool_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("ethtool");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
