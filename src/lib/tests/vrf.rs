// SPDX-License-Identifier: Apache-2.0

use nispor::NetState;
use pretty_assertions::assert_eq;

use std::panic;

mod utils;

const IFACE_NAME: &str = "vrf0";

const EXPECTED_VRF_INFO: &str = r#"---
table_id: 10
subordinates:
  - eth1
  - eth2"#;

#[test]
#[ignore] // Github Action does not have VRF supported
fn test_get_vrf_iface_yaml() {
    with_vrf_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, nispor::IfaceType::Vrf);
        assert_eq!(
            serde_yaml::to_string(&iface.vrf).unwrap().trim(),
            EXPECTED_VRF_INFO
        );
    });
}

fn with_vrf_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    utils::set_network_environment("vrf");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
