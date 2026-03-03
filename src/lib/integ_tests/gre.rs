// SPDX-License-Identifier: Apache-2.0

use pretty_assertions::assert_eq;

const IFACE_NAME: &str = "gretap99";

// This test ensures gretap interfaces are correctly identified as
// Other("gretap") rather than being treated as ethernet.
#[test]
fn test_gretap_iface_type() {
    with_gretap(|| {
        let state = crate::NetState::retrieve()
            .expect("Failed to retrieve network state");
        let iface = &state.ifaces[IFACE_NAME];

        assert_eq!(iface.iface_type, crate::IfaceType::Other("gretap".into()));
    })
}

fn with_gretap<T>(test: T)
where
    T: FnOnce() + std::panic::UnwindSafe,
{
    super::utils::set_network_environment("gretap");

    let result = std::panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    if let Err(e) = result {
        std::panic::resume_unwind(e);
    }
}
