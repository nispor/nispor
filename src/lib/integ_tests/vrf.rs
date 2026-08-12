// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{ErrorKind, IfaceState, NetConf, NetState};

const IFACE_NAME: &str = "vrf0";

const EXPECTED_VRF_INFO: &str = r#"---
table-id: 10
ports:
  - eth1
  - eth2"#;

const VRF_ENV_CREATE_YML: &str = r#"---
interfaces:
  - name: vrf0
    type: vrf
    vrf:
      table-id: 10
  - name: eth1
    type: veth
    veth:
      peer: eth1.ep
    controller: vrf0
  - name: eth2
    type: veth
    veth:
      peer: eth2.ep
    controller: vrf0
"#;

const VRF_ENV_DELETE_YML: &str = r#"---
interfaces:
  - name: vrf0
    type: vrf
    state: absent
  - name: eth1
    type: veth
    state: absent
  - name: eth2
    type: veth
    state: absent
"#;

#[test]
fn test_get_vrf_iface_yaml() {
    with_vrf_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vrf);
        assert_value_match(EXPECTED_VRF_INFO, &iface.vrf);
    });
}

fn with_vrf_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(VRF_ENV_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(VRF_ENV_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    assert!(result.is_ok())
}

const VRF_CREATE_YML: &str = r#"---
interfaces:
  - name: vrf0
    type: vrf
    vrf:
      table-id: 10
  - name: dummy1
    type: dummy
    controller: vrf0
  - name: dummy2
    type: dummy
    controller: vrf0
"#;

const VRF_DELETE_YML: &str = r#"---
interfaces:
  - name: vrf0
    type: vrf
    state: absent
  - name: dummy1
    type: dummy
    state: absent
  - name: dummy2
    type: dummy
    state: absent
"#;

const VRF_CHANGE_TABLE_YML: &str = r#"---
interfaces:
  - name: vrf0
    type: vrf
    vrf:
      table-id: 20
"#;

#[test]
fn test_create_and_delete_vrf() {
    let net_conf: NetConf = serde_yaml::from_str(VRF_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        let state = NetState::retrieve().unwrap();
        let vrf0 = &state.ifaces[IFACE_NAME];
        assert_eq!(vrf0.iface_type, crate::IfaceType::Vrf);
        assert_eq!(vrf0.state, IfaceState::Up);
        assert_eq!(vrf0.vrf.as_ref().unwrap().table_id, 10);
        assert_eq!(
            vrf0.vrf.as_ref().unwrap().ports,
            vec!["dummy1".to_string(), "dummy2".to_string()]
        );
        assert_eq!(
            state.ifaces["dummy1"].controller.as_deref(),
            Some(IFACE_NAME)
        );
        assert_eq!(
            state.ifaces["dummy2"].controller.as_deref(),
            Some(IFACE_NAME)
        );
    });

    let net_conf: NetConf = serde_yaml::from_str(VRF_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
    assert!(result.is_ok())
}

#[test]
fn test_reapply_vrf_conf() {
    let net_conf: NetConf = serde_yaml::from_str(VRF_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    // Re-applying the same configuration must succeed as a no-op.
    let net_conf: NetConf = serde_yaml::from_str(VRF_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        let state = NetState::retrieve().unwrap();
        let vrf0 = &state.ifaces[IFACE_NAME];
        assert_eq!(vrf0.iface_type, crate::IfaceType::Vrf);
        assert_eq!(vrf0.vrf.as_ref().unwrap().table_id, 10);
    });

    let net_conf: NetConf = serde_yaml::from_str(VRF_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    assert!(result.is_ok())
}

#[test]
fn test_change_vrf_table_id_rejected() {
    let net_conf: NetConf = serde_yaml::from_str(VRF_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let net_conf: NetConf = serde_yaml::from_str(VRF_CHANGE_TABLE_YML).unwrap();
    let err = net_conf.apply().unwrap_err();
    assert!(matches!(err.kind, ErrorKind::InvalidArgument));
    assert!(err.msg.contains("delete and recreate"));

    let net_conf: NetConf = serde_yaml::from_str(VRF_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
}
