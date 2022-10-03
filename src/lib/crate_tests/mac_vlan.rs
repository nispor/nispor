// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "mac0";

const EXPECTED_MAC_VLAN_STATE: &str = r#"---
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
        assert_eq!(iface.iface_type, crate::IfaceType::MacVlan);
        assert_value_match(EXPECTED_MAC_VLAN_STATE, &iface.mac_vlan);
    });
}

fn with_macvlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("macvlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
