// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{NetConf, NetState};

const IFACE_NAME: &str = "br0";
const PORT1_NAME: &str = "dummy1";
const PORT2_NAME: &str = "dummy2";

const EXPECTED_PORT1_BRIDGE_INFO: &str = r#"---
vlans:
  - vid: 1
    is_pvid: false
    is_egress_untagged: true
  - vid: 10
    is_pvid: true
    is_egress_untagged: true"#;

const EXPECTED_PORT2_BRIDGE_INFO: &str = r#"---
vlans:
  - vid: 1
    is_pvid: true
    is_egress_untagged: true
  - vid_range:
      - 2
      - 4094
    is_pvid: false
    is_egress_untagged: false"#;

static BR_SELF_VLAN: &str = r#"
  - vid: 1
    is_pvid: false
    is_egress_untagged: true
  - vid: 11
    is_pvid: true
    is_egress_untagged: true"#;

#[test]
fn test_get_br_vlan_filter_iface_yaml() {
    with_br_with_vlan_filter_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get(IFACE_NAME).unwrap();
        if let Some(bridge_info) = &iface.bridge {
            assert_eq!(bridge_info.vlan_filtering, Some(true))
        }
        assert_value_match(BR_SELF_VLAN, iface.bridge_vlan.as_ref().unwrap());

        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_value_match(EXPECTED_PORT1_BRIDGE_INFO, &port1.bridge_port);
        assert_value_match(EXPECTED_PORT2_BRIDGE_INFO, &port2.bridge_port);
    });
}

const BRIDGE_CREATE_YML: &str = r#"---
interfaces:
  - name: br0
    type: bridge
    mac-address: 00:23:45:67:89:1c
    bridge:
      stp_state: disabled
      vlan_filtering: true
      vlans:
      - vid: 1
        is_pvid: false
        is_egress_untagged: true
      - vid: 11
        is_pvid: true
        is_egress_untagged: true
  - name: dummy1
    type: dummy
    state: up
    controller: br0
    bridge-port:
      vlans:
      - vid: 1
        is_pvid: false
        is_egress_untagged: true
      - vid: 10
        is_pvid: true
        is_egress_untagged: true
  - name: dummy2
    type: dummy
    state: up
    controller: br0
    bridge-port:
      vlans:
        - vid: 1
          is_pvid: true
          is_egress_untagged: true
        - vid_range:
          - 2
          - 4094
          is_pvid: false
          is_egress_untagged: false"#;

const BRIDGE_DELETE_YML: &str = r#"---
interfaces:
  - name: br0
    type: bridge
    state: absent
  - name: dummy1
    type: dummy
    state: absent
  - name: dummy2
    type: dummy
    state: absent"#;

fn with_br_with_vlan_filter_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(BRIDGE_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    assert!(result.is_ok())
}

const BRIDGE_VLAN_MODIFY_YML: &str = r#"---
interfaces:
  - name: br0
    type: bridge
    bridge:
      stp_state: disabled
      vlan_filtering: true
      vlans:
      - vid: 1
        is_pvid: false
        is_egress_untagged: true
      - vid: 11
        is_pvid: true
        is_egress_untagged: true
        remove: true
      - vid: 21
        is_pvid: true
        is_egress_untagged: true
  - name: dummy1
    type: dummy
    state: up
    controller: br0
    bridge-port:
      vlans:
      - vid: 1
        is_pvid: false
        is_egress_untagged: true
      - vid: 10
        is_pvid: true
        is_egress_untagged: true
        remove: true
      - vid: 20
        is_pvid: true
        is_egress_untagged: true
  - name: dummy2
    type: dummy
    state: up
    controller: br0
    bridge-port:
      vlans:
        - vid: 1
          is_pvid: true
          is_egress_untagged: true
        - vid_range:
          - 2
          - 4094
          is_pvid: false
          is_egress_untagged: false
          remove: true
        - vid_range:
          - 4
          - 4094
          is_pvid: false
          is_egress_untagged: false"#;

const NEW_PORT1_BRIDGE_INFO: &str = r#"---
vlans:
  - vid: 1
    is_pvid: false
    is_egress_untagged: true
  - vid: 20
    is_pvid: true
    is_egress_untagged: true"#;

const NEW_PORT2_BRIDGE_INFO: &str = r#"---
vlans:
  - vid: 1
    is_pvid: true
    is_egress_untagged: true
  - vid_range:
      - 4
      - 4094
    is_pvid: false
    is_egress_untagged: false"#;

static NEW_BR_SELF_VLAN: &str = r#"
  - vid: 1
    is_pvid: false
    is_egress_untagged: true
  - vid: 21
    is_pvid: true
    is_egress_untagged: true"#;

#[test]
fn test_modify_bridge_vlan() {
    with_br_with_vlan_filter_iface(|| {
        let net_conf: NetConf =
            serde_yaml::from_str(BRIDGE_VLAN_MODIFY_YML).unwrap();
        net_conf.apply().unwrap();

        let state = NetState::retrieve().unwrap();

        let iface = &state.ifaces[IFACE_NAME];
        assert_value_match(
            NEW_BR_SELF_VLAN,
            iface.bridge_vlan.as_ref().unwrap(),
        );

        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_value_match(NEW_PORT1_BRIDGE_INFO, &port1.bridge_port);
        assert_value_match(NEW_PORT2_BRIDGE_INFO, &port2.bridge_port);
    })
}
