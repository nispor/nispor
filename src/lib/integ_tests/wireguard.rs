// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::{NetConf, NetState};

const IFACE_NAME: &str = "wg_nispor_test";

const WIREGUARD_CREATE_YAML: &str = r#"
interfaces:
  - name: wg_nispor_test
    type: wireguard
    wireguard:
      private-key: "6LTHiAM4vgKEgi5vm30f/EBIEWFDmySkTc9EWCcIqEs="
      peers:
        - endpoint: 192.0.2.254:9999
          public-key: "8bdQrVLqiw3ZoHCucNh1YfH0iCWuyStniRr8t7H24Fk="
          preshared-key: "iDMSzj8ZMpdbVWuVjOm/f/Zzl7s1WOdhlJtdV0YXHEU="
          persistent-keepalive: 360
          allowed-ips:
            - address: 0.0.0.0
              prefix-length: 0
            - address: "::"
              prefix-length: 0
"#;

const WIREGUARD_CHANGE_YAML: &str = r#"
interfaces:
  - name: wg_nispor_test
    type: wireguard
    wireguard:
      private-key: "6LTHiAM4vgKEgi5vm30f/EBIEWFDmySkTc9EWCcIqEs="
      peers:
        - endpoint: 192.0.2.253:9999
          public-key: "8bdQrVLqiw3ZoHCucNh1YfH0iCWuyStniRr8t7H24Fk="
          preshared-key: "iDMSzj8ZMpdbVWuVjOm/f/Zzl7s1WOdhlJtdV0YXHEU="
          persistent-keepalive: 0
          allowed-ips:
            - address: 0.0.0.0
              prefix-length: 0
            - address: "::"
              prefix-length: 0
"#;

const WIREGUARD_DELETE_YML: &str = r#"---
interfaces:
  - name: wg_nispor_test
    type: wireguard
    state: absent
"#;

const EXPECTED_WIREGUARD_INFO: &str = r#"---
public-key: "JKossUAjywXuJ2YVcaeD6PaHs+afPmIthDuqEVlspwA="
peers:
- endpoint: 192.0.2.254:9999
  public-key: "8bdQrVLqiw3ZoHCucNh1YfH0iCWuyStniRr8t7H24Fk="
  persistent-keepalive: 360
  allowed-ips:
    - address: 0.0.0.0
      prefix-length: 0
    - address: "::"
      prefix-length: 0
"#;

const EXPECTED_CHANGED_WIREGUARD_INFO: &str = r#"---
public-key: "JKossUAjywXuJ2YVcaeD6PaHs+afPmIthDuqEVlspwA="
peers:
- endpoint: 192.0.2.253:9999
  public-key: "8bdQrVLqiw3ZoHCucNh1YfH0iCWuyStniRr8t7H24Fk="
  persistent-keepalive: 0
  allowed-ips:
    - address: 0.0.0.0
      prefix-length: 0
    - address: "::"
      prefix-length: 0
"#;

#[test]
fn test_create_change_and_delete_wireguard() {
    with_wireguard_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Wireguard);
        assert_value_match(EXPECTED_WIREGUARD_INFO, &iface.wireguard);

        let net_conf: NetConf =
            serde_yaml::from_str(WIREGUARD_CHANGE_YAML).unwrap();
        net_conf.apply().unwrap();

        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        assert_eq!(iface.iface_type, crate::IfaceType::Wireguard);
        assert_value_match(EXPECTED_CHANGED_WIREGUARD_INFO, &iface.wireguard);
    });
}

fn with_wireguard_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    let net_conf: NetConf =
        serde_yaml::from_str(WIREGUARD_CREATE_YAML).unwrap();
    net_conf.apply().unwrap();

    let result = panic::catch_unwind(|| {
        test();
    });

    let net_conf: NetConf = serde_yaml::from_str(WIREGUARD_DELETE_YML).unwrap();
    net_conf.apply().unwrap();

    assert!(result.is_ok())
}
