// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{NetConf, NetState};

const IFACE_NAME: &str = "dummy1.99";

const VLAN_CREATE_FULL_INFO_YAML: &str = r#"
interfaces:
  - name: dummy1
    type: dummy
  - name: dummy1.99
    type: vlan
    vlan:
      base-iface: dummy1
      vlan-id: 99
      protocol: 802.1q
      is-reorder-hdr: true
      is-gvrp: true
      is-loose-binding: true
      is-mvrp: false
      is-bridge-binding: true
      ingress-qos-map:
      - from: 2
        to: 9
      - from: 3
        to: 8
      - from: 4
        to: 7
      egress-qos-map:
      - from: 7
        to: 4
      - from: 8
        to: 3
      - from: 9
        to: 2
"#;

const VLAN_CHANGE_FULL_INFO_YAML: &str = r#"
interfaces:
  - name: dummy1.99
    type: vlan
    vlan:
      is-reorder-hdr: false
      is-gvrp: false
      is-loose-binding: false
      is-mvrp: false
      is-bridge-binding: false
"#;

const REGR_IFACE_NAME: &str = "dummy2.98";

const REGR_VLAN_CREATE_YAML: &str = r#"
interfaces:
  - name: dummy2
    type: dummy
  - name: dummy2.98
    type: vlan
    vlan:
      base-iface: dummy2
      vlan-id: 98
"#;

// Regression test: modifying an existing VLAN's IP while omitting
// immutable vlan attributes (as a well-behaved caller should).
const REGR_VLAN_CHANGE_WITH_ATTRS_YAML: &str = r#"
interfaces:
  - name: dummy2.98
    type: vlan
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix-len: 24
"#;

// Regression test: modifying an existing VLAN's IP without any vlan section.
const REGR_VLAN_CHANGE_IP_ONLY_YAML: &str = r#"
interfaces:
  - name: dummy2.98
    type: vlan
    ipv4:
      addresses:
        - address: "192.0.2.2"
          prefix-len: 24
"#;

const REGR_VLAN_DELETE_YAML: &str = r#"---
interfaces:
  - name: dummy2.98
    type: vlan
    state: absent
  - name: dummy2
    type: dummy
    state: absent
"#;

const VLAN_DELETE_YML: &str = r#"---
interfaces:
  - name: dummy1.99
    type: vlan
    state: absent
  - name: dummy1
    type: dummy
    state: absent
"#;

const EXPECTED_VLAN_INFO: &str = r#"---
vlan-id: 99
protocol: 802.1q
base-iface: dummy1
is-reorder-hdr: true
is-gvrp: true
is-loose-binding: true
is-mvrp: false
is-bridge-binding: true
ingress-qos-map:
- from: 2
  to: 9
- from: 3
  to: 8
- from: 4
  to: 7
egress-qos-map:
- from: 7
  to: 4
- from: 8
  to: 3
- from: 9
  to: 2
"#;

const EXPECTED_CHANGED_VLAN_INFO: &str = r#"---
vlan-id: 99
protocol: 802.1q
base-iface: dummy1
is-reorder-hdr: false
is-gvrp: false
is-loose-binding: false
is-mvrp: false
is-bridge-binding: false
ingress-qos-map:
- from: 2
  to: 9
- from: 3
  to: 8
- from: 4
  to: 7
egress-qos-map:
- from: 7
  to: 4
- from: 8
  to: 3
- from: 9
  to: 2
"#;

#[test]
fn test_create_change_and_delete_vlan() {
    with_vlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        assert_value_match(EXPECTED_VLAN_INFO, &iface.vlan);

        let net_conf: NetConf =
            serde_yaml::from_str(VLAN_CHANGE_FULL_INFO_YAML).unwrap();
        net_conf.apply().unwrap();

        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        assert_value_match(EXPECTED_CHANGED_VLAN_INFO, &iface.vlan);
    });
}

// Regression: modifying an existing VLAN should not cause EINVAL.
// Tests both re-specifying immutable attrs and IP-only changes.
#[test]
fn test_modify_existing_vlan() {
    with_regr_vlan_iface(|| {
        // Changing IP while omitting immutable vlan attributes
        let net_conf: NetConf =
            serde_yaml::from_str(REGR_VLAN_CHANGE_WITH_ATTRS_YAML).unwrap();
        net_conf.apply().unwrap();

        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[REGR_IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        let ipv4 = iface.ipv4.as_ref().unwrap();
        assert!(
            ipv4.addresses
                .iter()
                .any(|a| a.address == "192.0.2.1" && a.prefix_len == 24)
        );

        // Changing only the IP without any vlan section
        let net_conf: NetConf =
            serde_yaml::from_str(REGR_VLAN_CHANGE_IP_ONLY_YAML).unwrap();
        net_conf.apply().unwrap();

        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[REGR_IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        let ipv4 = iface.ipv4.as_ref().unwrap();
        assert!(
            ipv4.addresses
                .iter()
                .any(|a| a.address == "192.0.2.2" && a.prefix_len == 24)
        );
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

fn with_regr_vlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf =
        serde_yaml::from_str(REGR_VLAN_CREATE_YAML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf =
        serde_yaml::from_str(REGR_VLAN_DELETE_YAML).unwrap();
    net_conf.apply().unwrap();

    assert!(result.is_ok())
}
