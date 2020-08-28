use nispor::NetState;

use std::panic;
use std::thread::sleep;
use std::time;

mod utils;

const IFACE_TEST: &str = "eth1";
const IFACE_MAC_TEST: &str = "AA:BB:CC:DD:EE:FF";

#[test]
fn test_get_iface_name() {
    with_eth1_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_TEST];
            let iface_name = &iface.name;
            assert_eq!(iface_name, IFACE_TEST);
        }
    });
}

#[test]
fn test_get_iface_mtu() {
    with_eth1_iface(|| {
        utils::cmd_exec(
            "ip",
            vec!["link", "set", "dev", IFACE_TEST, "mtu", "1000"],
        );
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_TEST];
            let mtu = iface.mtu;
            assert_eq!(mtu, 1000);
        }
    });
}

#[test]
fn test_get_iface_type() {
    with_eth1_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_TEST];
            let iface_type = &iface.iface_type;
            assert_eq!(iface_type, &nispor::IfaceType::Veth);
        }
    });
}

#[test]
fn test_get_iface_mac() {
    with_eth1_iface(|| {
        utils::cmd_exec(
            "ip",
            vec!["link", "set", "dev", IFACE_TEST, "address", IFACE_MAC_TEST],
        );
        sleep(time::Duration::from_millis(2000));
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_TEST];
            let mac = &iface.mac_address;
            assert_eq!(mac, IFACE_MAC_TEST);
        }
    });
}

#[test]
fn test_get_iface_state_down() {
    with_eth1_iface(|| {
        if let Ok(net_state) = NetState::retrieve() {
            let iface = &net_state.ifaces[IFACE_TEST];
            let state = &iface.state;
            assert_eq!(state, &nispor::IfaceState::Down);
        }
    });
}

#[test]
fn test_get_iface_state_up() {
    with_eth1_iface(|| {
        utils::cmd_exec("ip", vec!["link", "set", IFACE_TEST, "up"]);
        if let Ok(net_state) = NetState::retrieve() {
            let iface = &net_state.ifaces[IFACE_TEST];
            let state = &iface.state;
            assert_eq!(state, &nispor::IfaceState::Up);
        }
    });
}

fn with_eth1_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("veth");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
