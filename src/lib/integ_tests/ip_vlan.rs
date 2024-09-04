// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "ipvlan0";

const EXPECTED_IP_VLAN_STATE: &str = r#"---
base_iface: eth1
mode: l2
flags: []"#;

#[test]
fn test_get_ip_vlan_iface_yaml() {
    with_ip_vlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::IpVlan);
        assert_value_match(EXPECTED_IP_VLAN_STATE, &iface.ip_vlan);
    });
}

fn with_ip_vlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("ipvlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
