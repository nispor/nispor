// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{BondMode, NetConf, NetState};

const IFACE_NAME: &str = "bond99";
const PORT1_NAME: &str = "dummy1";
const PORT2_NAME: &str = "dummy2";

const EXPECTED_BOND_IFACE: &str = r#"---
name: bond99
iface_type: bond
bond:
  ports:
  - dummy1
  - dummy2
  mode: active-backup
  miimon: 30
  updelay: 60
  downdelay: 90
  use_carrier: true
  arp_interval: 0
  arp_all_targets: any
  arp_validate: none
  primary_reselect: always
  resend_igmp: 1
  all_ports_active: dropped
  min_links: 0
  lp_interval: 1
  peer_notif_delay: 0
  "#;

const EXPECTED_PORT1_INFO: &str = r#"---
port_state: active
mii_status: link_up
link_failure_count: 0
perm_hwaddr: "00:23:45:67:89:1a"
prio: -10
queue_id: 1"#;

const EXPECTED_PORT2_INFO: &str = r#"---
port_state: backup
mii_status: link_up
link_failure_count: 0
perm_hwaddr: "00:23:45:67:89:1b"
prio: -20
queue_id: 2"#;

const BOND_CREATE_YML: &str = r#"---
interfaces:
  - name: bond99
    type: bond
    bond:
      mode: active-backup
      miimon: 30
      updelay: 60
      downdelay: 90
      use_carrier: true
      arp_interval: 0
      arp_all_targets: any
      arp_validate: none
      primary_reselect: always
      resend_igmp: 1
      all_ports_active: dropped
      min_links: 0
      lp_interval: 1
      peer_notif_delay: 0
  - name: dummy1
    type: dummy
    controller: bond99
    mac-address: 00:23:45:67:89:1a
    bond-port:
      prio: -10
      queue_id: 1
  - name: dummy2
    type: dummy
    state: up
    controller: bond99
    mac-address: 00:23:45:67:89:1b
    bond-port:
      prio: -20
      queue_id: 2"#;

const BOND_PORT_REMOVE_YML: &str = r#"---
interfaces:
  - name: dummy1
    type: dummy
    controller: ""
  - name: dummy2
    type: dummy
    controller: """#;

const BOND_DELETE_YML: &str = r#"---
interfaces:
  - name: bond99
    state: absent
  - name: dummy1
    state: absent
  - name: dummy2
    state: absent"#;

const BOND_CHANGE_YML: &str = r#"---
interfaces:
  - name: bond99
    type: bond
    bond:
      miimon: 0
      arp_interval: 30
"#;

fn with_bond_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(BOND_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    // Wait miimon finish
    std::thread::sleep(std::time::Duration::from_secs(1));

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(BOND_DELETE_YML).unwrap();
    net_conf.apply().unwrap();
    let state = NetState::retrieve().unwrap();
    assert_eq!(None, state.ifaces.get(IFACE_NAME));
    assert!(result.is_ok())
}

#[test]
fn test_create_delete_bond() {
    with_bond_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let port1 = &state.ifaces[PORT1_NAME];
        let port2 = &state.ifaces[PORT2_NAME];
        assert_value_match(EXPECTED_BOND_IFACE, &iface);

        assert_value_match(EXPECTED_PORT1_INFO, &port1.bond_port);
        assert_value_match(EXPECTED_PORT2_INFO, &port2.bond_port);
        assert_eq!(port1.controller, Some("bond99".to_string()));
        assert_eq!(port2.controller, Some("bond99".to_string()));
        assert_eq!(port1.controller_type, Some(crate::ControllerType::Bond));
        assert_eq!(port2.controller_type, Some(crate::ControllerType::Bond));

        let net_conf: NetConf =
            serde_yaml::from_str(BOND_PORT_REMOVE_YML).unwrap();
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(&iface.iface_type, &crate::IfaceType::Bond);
        let empty_vec: Vec<String> = Vec::new();
        assert_eq!(&iface.bond.as_ref().unwrap().ports, &empty_vec);
    });
}

#[test]
fn test_change_bond_disable_miimon_enable_arp_interval() {
    with_bond_iface(|| {
        let net_conf: NetConf = serde_yaml::from_str(BOND_CHANGE_YML).unwrap();
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(&iface.iface_type, &crate::IfaceType::Bond);
        assert_eq!(iface.bond.as_ref().unwrap().miimon, Some(0));
        assert_eq!(iface.bond.as_ref().unwrap().updelay, Some(0));
        assert_eq!(iface.bond.as_ref().unwrap().downdelay, Some(0));
        assert_eq!(iface.bond.as_ref().unwrap().arp_interval, Some(30));
    })
}

const CHANGE_BOND_MODE_YAML: &str = r#"---
interfaces:
  - name: dummy1
    state: up
    controller: ""
  - name: dummy2
    state: up
    controller: ""
  - name: bond99
    type: bond
    state: up
    bond:
      mode: balance-rr"#;

#[test]
fn test_change_bond_mode() {
    with_bond_iface(|| {
        let net_conf: NetConf =
            serde_yaml::from_str(CHANGE_BOND_MODE_YAML).unwrap();
        net_conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(&iface.iface_type, &crate::IfaceType::Bond);
        assert!(&iface.flags.contains(&crate::IfaceFlag::Up));
        assert_eq!(
            iface.bond.as_ref().unwrap().mode,
            BondMode::BalanceRoundRobin
        );
    })
}
