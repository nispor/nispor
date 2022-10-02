// SPDX-License-Identifier: Apache-2.0

use nispor::NetState;
use pretty_assertions::assert_eq;
use std::panic;

mod utils;

const IFACE_NAME: &str = "dummy1";

#[test]
fn test_get_iface_dummy_yaml() {
    with_dummy_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &nispor::IfaceType::Dummy);
    });
}

fn with_dummy_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    utils::set_network_environment("dummy");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
