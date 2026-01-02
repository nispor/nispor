// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::{NetConf, NetState};

const IFACE_NAME: &str = "dummy1";

const DUMMY_CREATE_YML: &str = r#"---
interfaces:
  - name: dummy1
    type: dummy
    "#;

const DUMMY_DELETE_YML: &str = r#"---
interfaces:
  - name: dummy1
    type: veth
    state: absent"#;

#[test]
fn test_get_iface_dummy_yaml() {
    with_dummy_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Dummy);
    });
}

fn with_dummy_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf = serde_yaml::from_str(DUMMY_CREATE_YML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(DUMMY_DELETE_YML).unwrap();
    net_conf.apply().ok();
    assert!(result.is_ok())
}
