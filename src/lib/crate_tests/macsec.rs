// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use crate::NetState;

use super::utils::assert_value_match;

const IFACE_NAME: &str = "macsec0";

// SCI has been excluded from the state as it is non-deterministic
const EXPECTED_MACSEC_INFO: &str = r#"---
name: macsec0
iface_type: mac_sec
macsec:
  port: 0
  icv_len: 16
  cipher: gcm-aes128
  window: 0
  encoding_sa: 0
  encrypt: true
  protect: true
  send_sci: true
  end_station: false
  scb: false
  replay_protect: false
  validate: strict
  base_iface: eth1"#;

#[test]
fn test_get_macsec_iface_yaml() {
    with_macsec_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];

        assert_eq!(iface.iface_type, crate::IfaceType::MacSec);

        assert_value_match(EXPECTED_MACSEC_INFO, iface);
    });
}

fn with_macsec_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("macsec");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
