// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

#[test]
fn test_iface_info_loopback() {
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces["lo"];
    assert_eq!(iface.iface_type, crate::IfaceType::Loopback);
    assert_eq!(iface.state, crate::IfaceState::Unknown);
    assert_eq!(iface.mtu, 65536);
    assert_eq!(&iface.mac_address, "00:00:00:00:00:00");
    assert_eq!(iface.max_mtu, None);
    assert_eq!(iface.min_mtu, None);
    assert_eq!(iface.driver, None); // loopback device is driver-less
    assert_eq!(
        iface.flags,
        &[
            crate::IfaceFlag::Loopback,
            crate::IfaceFlag::LowerUp,
            crate::IfaceFlag::Running,
            crate::IfaceFlag::Up,
        ]
    );
}
