// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::NetState;

const IFACE_NAME: &str = "vxlan0";

const EXPECTED_VXLAN_INFO: &str = r#"---
remote: 8.8.8.8
vxlan-id: 101
base-iface: eth1
local: 1.1.1.1
ttl: 0
tos: 0
learning: true
ageing: 300
max-address: 0
src-port-min: 0
src-port-max: 0
proxy: false
rsc: false
l2miss: false
l3miss: false
dst-port: 4789
udp-check-sum: true
udp6-zero-check-sum-tx: false
udp6-zero-check-sum-rx: false
remote-check-sum-tx: false
remote-check-sum-rx: false
gbp: false
remote-check-sum-no-partial: false
collect-metadata: false
label: 0
gpe: false
ttl-inherit: false
df: 0"#;

#[test]
fn test_get_vxlan_iface_yaml() {
    with_vxlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vxlan);
        assert_value_match(EXPECTED_VXLAN_INFO, &iface.vxlan);
    });
}

fn with_vxlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("vxlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
