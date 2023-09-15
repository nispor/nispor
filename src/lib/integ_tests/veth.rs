// SPDX-License-Identifier: Apache-2.0

use crate::{IfaceState, NetConf, NetState};
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "veth1";

const EXPECTED_VETH_INFO: &str = r#"---
peer: veth1.ep"#;

#[test]
fn test_get_veth_iface_yaml() {
    with_veth_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_value_match(EXPECTED_VETH_INFO, &iface.veth);
    });
}

fn with_veth_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("veth");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

const VETH_CREATE_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    mac_address: 00:23:45:67:89:1a
    veth:
      peer: veth1.ep
  - name: veth1.ep
    type: veth"#;

const VETH_CHANGE_MAC_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    mac_address: 00:23:45:67:89:2a"#;

const VETH_DOWN_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    state: down"#;

const VETH_DELETE_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    state: absent"#;

#[test]
fn test_create_down_delete_veth() {
    let net_conf: NetConf = serde_yaml::from_str(VETH_CREATE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(&iface.iface_type, &crate::IfaceType::Veth);
    assert_eq!(iface.veth.as_ref().unwrap().peer, "veth1.ep");
    assert_eq!(iface.state, IfaceState::Up);
    assert_eq!(iface.mac_address, "00:23:45:67:89:1a".to_string());

    // Change the MAC should have the interface as UP state
    let net_conf: NetConf = serde_yaml::from_str(VETH_CHANGE_MAC_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(iface.state, IfaceState::Up);
    assert_eq!(iface.mac_address, "00:23:45:67:89:2a".to_string());

    let net_conf: NetConf = serde_yaml::from_str(VETH_DOWN_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(iface.state, IfaceState::Down);

    let net_conf: NetConf = serde_yaml::from_str(VETH_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
}
