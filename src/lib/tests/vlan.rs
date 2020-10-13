extern crate nispor;

use std::panic;

mod utils;

const IFACE_STATE: &str = "eth1.101";

#[test]
fn test_get_iface_vlan_type() {
    with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let iface_type = &iface.iface_type;
            assert_eq!(iface_type, &nispor::IfaceType::Vlan)
        }
    });
}

#[test]
fn test_get_vlan_id() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(101, vlan_state.unwrap().vlan_id)
        }
    });
}

#[test]
fn test_get_vlan_base_iface() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!("eth1", vlan_state.unwrap().base_iface)
        }
    });
}

#[test]
fn test_get_vlan_is_protocol_802_1_q() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                nispor::VlanProtocol::Ieee8021Q,
                vlan_state.unwrap().protocol
            )
        }
    });
}

#[test]
fn test_get_vlan_is_protocol_802_1_ad() {
     with_vlan_eth1_101_iface("proto 802.1ad", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                nispor::VlanProtocol::Ieee8021AD,
                vlan_state.unwrap().protocol
            )
        }
    });
}

#[test]
fn test_get_vlan_is_reorder_hdr_true() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                true,
                vlan_state.unwrap().is_reorder_hdr
            )
        }
    });
}

#[test]
fn test_get_vlan_is_reorder_hdr_false() {
     with_vlan_eth1_101_iface("reorder_hdr off", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                false,
                vlan_state.unwrap().is_reorder_hdr
            )
        }
    });
}

#[test]
fn test_get_vlan_is_gvrp_false() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                false,
                vlan_state.unwrap().is_gvrp
            )
        }
    });
}

#[test]
fn test_get_vlan_is_gvrp_true() {
     with_vlan_eth1_101_iface("gvrp on", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                true,
                vlan_state.unwrap().is_gvrp
            )
        }
    });
}

#[test]
fn test_get_vlan_is_loose_binding_false() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                false,
                vlan_state.unwrap().is_loose_binding
            )
        }
    });
}

#[test]
fn test_get_vlan_is_loose_binding_true() {
     with_vlan_eth1_101_iface("loose_binding on", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                true,
                vlan_state.unwrap().is_loose_binding
            )
        }
    });
}

#[test]
fn test_get_vlan_is_mvrp_false() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                false,
                vlan_state.unwrap().is_mvrp
            )
        }
    });
}

#[test]
fn test_get_vlan_is_mvrp_true() {
     with_vlan_eth1_101_iface("mvrp on", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                true,
                vlan_state.unwrap().is_mvrp
            )
        }
    });
}

#[test]
fn test_get_vlan_is_bridge_binding_false() {
     with_vlan_eth1_101_iface("", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                false,
                vlan_state.unwrap().is_bridge_binding
            )
        }
    });
}

#[test]
fn test_get_vlan_is_bridge_binding_true() {
     with_vlan_eth1_101_iface("bridge_binding on", || {
        if let Ok(state) = nispor::get_state() {
            let iface = &state.ifaces[IFACE_STATE];
            let vlan_state = iface.vlan.as_ref();
            assert_eq!(
                true,
                vlan_state.unwrap().is_bridge_binding
            )
        }
    });
}
fn with_vlan_eth1_101_iface<T>(options: &str, test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("vlan", options);

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
