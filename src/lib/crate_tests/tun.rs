// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "tun1";

const EXPECTED_TUN_INFO: &str = r#"---
mode: tun
owner: 1001
group: 0
pi: false
vnet_hdr: true
multi_queue: true
persist: true
num_queues: 0
num_disabled_queues: 0"#;

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
