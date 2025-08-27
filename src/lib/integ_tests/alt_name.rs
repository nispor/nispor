// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::{NetConf, NetState};

const IFACE_NAME: &str = "veth1";

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

#[test]
fn test_add_and_remove_alt_name_bulk() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(
            r#"---
            ifaces:
              - name: veth1
                alt_names:
                  - name: port1
                  - name: internal"#,
        )
        .unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(
            iface.alt_names,
            vec!["port1".to_string(), "internal".to_string()]
        );

        let conf: NetConf = serde_yaml::from_str(
            r#"---
            ifaces:
              - name: veth1
                alt_names:
                  - name: port1
                    remove: true
                  - name: internal
                    remove: true"#,
        )
        .unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.alt_names, Vec::<String>::new());
    });
}

#[test]
fn test_add_and_remove_alt_name_mixed() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(
            r#"---
            ifaces:
              - name: veth1
                alt_names:
                  - name: port1
                  - name: internal"#,
        )
        .unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(
            iface.alt_names,
            vec!["port1".to_string(), "internal".to_string()]
        );

        let conf: NetConf = serde_yaml::from_str(
            r#"---
            ifaces:
              - name: veth1
                alt_names:
                  - name: internal
                    remove: true
                  - name: wan0"#,
        )
        .unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(
            iface.alt_names,
            vec!["port1".to_string(), "wan0".to_string()]
        );
    });
}
