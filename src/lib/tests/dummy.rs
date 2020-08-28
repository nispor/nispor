use nispor::NetState;

use std::panic;

mod utils;

const IFACE_STATE: &str = "dummy1";

#[test]
fn test_get_iface_dummy_type() {
    with_dummy1_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_STATE];
            let iface_type = &iface.iface_type;
            assert_eq!(iface_type, &nispor::IfaceType::Dummy)
        }
    });
}

fn with_dummy1_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("dummy");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
