// SPDX-License-Identifier: Apache-2.0

use crate::{NetConf, NetState};
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "bond99";
const PORT1_NAME: &str = "eth1";
const PORT2_NAME: &str = "eth2";

const EXPECTED_BOND_IFACE: &str = r#"---
name: bond99
iface_type: bond
bond:
  subordinates:
  - eth1
  - eth2
  mode: balance-rr
  miimon: 0
  updelay: 0
  downdelay: 0
  use_carrier: true
  arp_interval: 0
  arp_all_targets: any
  arp_validate: none
  primary_reselect: always
  fail_over_mac: none
  xmit_hash_policy: layer2
  resend_igmp: 1
  num_unsol_na: 1
  all_subordinates_active: dropped
  min_links: 0
  lp_interval: 1
  packets_per_subordinate: 1
  lacp_rate: slow
  ad_select: stable
  tlb_dynamic_lb: true
  peer_notif_delay: 0
  lacp_active: true
  "#;

const EXPECTED_PORT1_INFO: &str = r#"---
subordinate_state: active
mii_status: link_up
link_failure_count: 0
perm_hwaddr: "00:23:45:67:89:1a"
queue_id: 0"#;

const EXPECTED_PORT2_INFO: &str = r#"---
subordinate_state: active
mii_status: link_up
link_failure_count: 0
perm_hwaddr: "00:23:45:67:89:1b"
queue_id: 0"#;

#[test]
fn test_get_iface_bond_yaml() {
    with_bond_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];

        assert_value_match(EXPECTED_BOND_IFACE, &iface);

        assert_value_match(EXPECTED_PORT1_INFO, &port1.bond_subordinate);
        assert_value_match(EXPECTED_PORT2_INFO, &port2.bond_subordinate);
        assert_eq!(port1.controller, Some("bond99".to_string()));
        assert_eq!(port2.controller, Some("bond99".to_string()));
        assert_eq!(port1.controller_type, Some(crate::ControllerType::Bond));
        assert_eq!(port2.controller_type, Some(crate::ControllerType::Bond));
    });
}

fn with_bond_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("bond");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

const BOND_CREATE_YML: &str = r#"---
ifaces:
  - name: bond99
    type: bond
  - name: veth1
    type: veth
    controller: bond99
    veth:
      peer: veth1.ep
  - name: veth1.ep
    type: veth
    state: up"#;

const BOND_PORT_REMOVE_YML: &str = r#"---
ifaces:
  - name: veth1
    type: veth
    veth:
      peer: veth1.ep"#;

const BOND_DELETE_YML: &str = r#"---
ifaces:
  - name: bond99
    state: absent
  - name: veth1
    state: absent"#;

#[test]
fn test_create_delete_bond() {
    let net_conf: NetConf = serde_yaml::from_str(BOND_CREATE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(&iface.iface_type, &crate::IfaceType::Bond);
    assert_eq!(
        &iface.bond.as_ref().unwrap().subordinates,
        &vec!["veth1".to_string()]
    );

    let net_conf: NetConf = serde_yaml::from_str(BOND_PORT_REMOVE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    let iface = &state.ifaces[IFACE_NAME];
    assert_eq!(&iface.iface_type, &crate::IfaceType::Bond);
    let empty_vec: Vec<String> = Vec::new();
    assert_eq!(&iface.bond.as_ref().unwrap().subordinates, &empty_vec);

    let net_conf: NetConf = serde_yaml::from_str(BOND_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
}
