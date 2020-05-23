extern crate nispor;

use std::panic;
use std::thread::sleep;
use std::time;

mod utils;

const IFACE_TEST: &str = "dummy0";
const IFACE_MAC_TEST: &str = "AA:BB:CC:DD:EE:FF";

#[test]
fn test_get_iface_name() {
    with_dummy_iface(|| {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_TEST];
            let iface_name = &iface.name;
            assert_eq!(iface_name, IFACE_TEST);
        }
    });
}

#[test]
fn test_get_iface_mtu() {
    with_dummy_iface(|| {
        utils::cmd_exec(
            "ip",
            vec!["link", "set", "dev", IFACE_TEST, "mtu", "1000"],
        );
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_TEST];
            let mtu = iface.mtu;
            assert_eq!(mtu, 1000);
        }
    });
}

#[test]
fn test_get_iface_mac() {
    with_dummy_iface(|| {
        utils::cmd_exec(
            "ip",
            vec!["link", "set", "dev", IFACE_TEST, "address", IFACE_MAC_TEST],
        );
        sleep(time::Duration::from_millis(2000));
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_TEST];
            let mac = &iface.mac_address;
            assert_eq!(mac, IFACE_MAC_TEST);
        }
    });
}

fn with_dummy_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::create_dummy(IFACE_TEST);

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::delete_dummy(IFACE_TEST);
    assert!(result.is_ok())
}
