// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{NetConf, NetState};

const IFACE_NAME: &str = "vxlan1";

const EXPECTED_VXLAN_INFO: &str = r#"---
remote: 8.8.8.8
vxlan-id: 102
base-iface: dummy1
local: 1.1.1.1
ttl: 16
tos: 24
learning: true
ageing: 300
max-address: 0
src-port-min: 0
src-port-max: 0
proxy: true
rsc: true
l2miss: true
l3miss: true
dst-port: 4789
udp-check-sum: true
udp6-zero-check-sum-tx: false
udp6-zero-check-sum-rx: false
remote-check-sum-tx: true
remote-check-sum-rx: true
gbp: true
remote-check-sum-no-partial: false
collect-metadata: false
label: 0
gpe: false
ttl-inherit: false"#;

const VXLAN_DELETE_YAML: &str = r#"
interfaces:
  - name: vxlan1
    type: vxlan
    state: absent
  - name: dummy1
    type: dummy
    state: absent
"#;

const VXLAN_CREATE_YAML: &str = r#"
interfaces:
  - name: dummy1
    type: dummy
  - name: vxlan1
    type: vxlan
    vxlan:
      base-iface: dummy1
      vxlan-id: 102
      remote: 8.8.8.8
      local: 1.1.1.1
      dst-port: 4789
      learning: true
      ttl: 16
      tos: 24
      proxy: true
      l2miss: true
      rsc: true
      l3miss: true
      remote-check-sum-tx: true
      remote-check-sum-rx: true
      gbp: true
      remote-check-sum-no-partial: false
"#;

#[test]
fn test_create_and_delete_vxlan() {
    with_vxlan_conf(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Vxlan);
        assert_value_match(EXPECTED_VXLAN_INFO, &iface.vxlan);
    });
}

fn with_vxlan_conf<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(VXLAN_CREATE_YAML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(VXLAN_DELETE_YAML).unwrap();
    net_conf.apply().unwrap();

    assert!(result.is_ok())
}
