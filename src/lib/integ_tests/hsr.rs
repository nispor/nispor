// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::NetState;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "hsr0";

// seq_nr has been excluded as it is non-deterministic
const EXPECTED_HSR_INFO: &str = r#"---
name: hsr0
iface_type: hsr
hsr:
  port1: eth1
  port2: eth2
  supervision_addr: 01:15:4e:00:01:2d
  multicast_spec: 0
  version: 0
  protocol: prp"#;

#[test]
fn test_get_hsr_iface_yaml() {
    with_hsr_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];

        assert_eq!(iface.iface_type, crate::IfaceType::Hsr);

        assert_value_match(EXPECTED_HSR_INFO, iface);
    });
}

fn with_hsr_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("hsr");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
