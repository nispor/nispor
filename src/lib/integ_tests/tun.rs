// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::NetState;

const IFACE_NAME: &str = "tun1";

const EXPECTED_TUN_INFO: &str = r#"---
mode: tun
owner: 1001
group: 0
pi: false
vnet-hdr: true
multi-queue: true
persist: true
num-queues: 0
num-disabled-queues: 0"#;

#[test]
fn test_get_tun_iface_yaml() {
    with_tun_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Tun);
        assert_value_match(EXPECTED_TUN_INFO, &iface.tun);
    });
}

fn with_tun_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("tun");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
