// SPDX-License-Identifier: Apache-2.0

use crate::{NetConf, NetState};
use pretty_assertions::assert_eq;

use std::panic;

const IFACE_NAME: &str = "eth1.101";

const EXPECTED_VLAN_INFO: &str = r#"---
vlan_id: 101
protocol: 802.1q
base_iface: eth1
is_reorder_hdr: true
is_gvrp: false
is_loose_binding: false
is_mvrp: false
is_bridge_binding: false"#;

#[test]
fn test_get_vlan_iface_yaml() {
    with_vlan_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vlan);
        assert_eq!(
            serde_yaml::to_string(&iface.vlan).unwrap().trim(),
            EXPECTED_VLAN_INFO
        );
    });
}

fn with_vlan_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("vlan");

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
    veth:
      peer: veth1.ep
  - name: veth1.ep
    type: veth"#;

const VETH_DELETE_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    state: absent"#;

const VLAN_CREATE_YML: &str = r#"---
ifaces:
  - name: veth1.99
    type: vlan
    vlan:
      base_iface: veth1
      vlan_id: 99"#;

const VLAN_DELETE_YML: &str = r#"---
ifaces:
  - name: veth1.99
    type: vlan
    state: absent"#;

#[test]
fn test_create_delete_vlan() {
    let net_conf: NetConf = serde_yaml::from_str(VETH_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let net_conf: NetConf = serde_yaml::from_str(VLAN_CREATE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces["veth1.99"];
    assert_eq!(&iface.iface_type, &crate::IfaceType::Vlan);
    assert_eq!(iface.vlan.as_ref().unwrap().vlan_id, 99);
    assert_eq!(iface.vlan.as_ref().unwrap().base_iface.as_str(), "veth1");

    let net_conf: NetConf = serde_yaml::from_str(VLAN_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get("veth1.99"));

    let net_conf: NetConf = serde_yaml::from_str(VETH_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
}
