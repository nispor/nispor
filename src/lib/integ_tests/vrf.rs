// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "vrf0";

const EXPECTED_VRF_INFO: &str = r#"---
table_id: 10
subordinates:
  - eth1
  - eth2"#;

#[test]
fn test_get_vrf_iface_yaml() {
    with_vrf_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vrf);
        assert_value_match(EXPECTED_VRF_INFO, &iface.vrf);
    });
}

fn with_vrf_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("vrf");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
