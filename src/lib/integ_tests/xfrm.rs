// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::NetState;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "xfrm1";

const EXPECTED_XFRM_INFO: &str = r#"---
name: xfrm1
iface_type: xfrm
xfrm:
  base_iface: eth1
  iface_id: 99"#;

#[test]
fn test_get_xfrm_iface_yaml() {
    with_xfrm_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];

        assert_eq!(iface.iface_type, crate::IfaceType::Xfrm);

        assert_value_match(EXPECTED_XFRM_INFO, iface);
    });
}

fn with_xfrm_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("xfrm");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
