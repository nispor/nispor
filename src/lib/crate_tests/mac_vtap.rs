// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

const IFACE_NAME: &str = "macvtap0";

const EXPECTED_MAC_VTAP_INFO: &str = r#"---
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
        assert_eq!(iface.iface_type, crate::IfaceType::MacVtap);
        assert_eq!(
            serde_yaml::to_string(&iface.mac_vtap).unwrap().trim(),
            EXPECTED_MAC_VTAP_INFO
        );
    });
}

fn with_macvtap_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("macvtap");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
