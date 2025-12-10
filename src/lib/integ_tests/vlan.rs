// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{NetConf, NetState};

const IFACE_NAME: &str = "dummy1.99";

const VLAN_CREATE_FULL_INFO_YAML: &str = r#"
ifaces:
  - name: dummy1
    type: dummy
  - name: dummy1.99
    type: vlan
    vlan:
      base_iface: dummy1
      vlan_id: 99
      protocol: 802.1q
      is_reorder_hdr: true
      is_gvrp: true
      is_loose_binding: true
      is_mvrp: true
      is_bridge_binding: true
      ingress_qos_map:
      - from: 2
        to: 9
      - from: 3
        to: 8
      - from: 4
        to: 7
      egress_qos_map:
      - from: 7
        to: 4
      - from: 8
        to: 3
      - from: 9
        to: 2
"#;

const VLAN_DELETE_YML: &str = r#"---
ifaces:
  - name: dummy1.99
    type: vlan
    state: absent
  - name: dummy1
    type: dummy
    state: absent
"#;

const EXPECTED_VLAN_INFO: &str = r#"---
vlan_id: 99
protocol: 802.1q
base_iface: dummy1
is_reorder_hdr: true
is_gvrp: true
is_loose_binding: true
is_mvrp: true
is_bridge_binding: true
ingress_qos_map:
- from: 2
  to: 9
- from: 3
  to: 8
- from: 4
  to: 7
egress_qos_map:
- from: 7
  to: 4
- from: 8
  to: 3
- from: 9
  to: 2
"#;

#[test]
fn test_create_and_delete_vlan() {
    with_vlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        assert_value_match(EXPECTED_VLAN_INFO, &iface.vlan);
    });
}

fn with_vlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf =
        serde_yaml::from_str(VLAN_CREATE_FULL_INFO_YAML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(VLAN_DELETE_YML).unwrap();
    net_conf.apply().unwrap();

    assert!(result.is_ok())
}
