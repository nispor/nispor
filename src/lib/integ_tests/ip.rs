// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
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

const ADD_IP_CONF: &str = r#"---
ifaces:
  - name: veth1
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix_len: 24
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix_len: 64"#;

const ADD_IP_CONF_DYNAMIC: &str = r#"---
ifaces:
  - name: veth1
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix_len: 24
          valid_lft: 120sec
          preferred_lft: 60sec
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix_len: 64
          valid_lft: 121sec
          preferred_lft: 61sec"#;

const DEL_IP_CONF: &str = r#"---
ifaces:
  - name: veth1
    ipv4:
      addresses:
        - address: "192.0.2.1"
          prefix_len: 24
          remove: true
    ipv6:
      addresses:
        - address: "2001:db8:a::9"
          prefix_len: 64
          remove: true"#;

const EXPECTED_IPV4_INFO: &str = r#"---
addresses:
  - address: 192.0.2.1
    prefix_len: 24
    valid_lft: forever
    preferred_lft: forever"#;

const EXPECTED_IPV4_DYNAMIC_INFO: &str = r#"---
addresses:
  - address: 192.0.2.1
    prefix_len: 24
    valid_lft: 115sec
    preferred_lft: 55sec"#;

const EXPECTED_IPV6_INFO: &str = r#"---
addresses:
  - address: "2001:db8:a::9"
    prefix_len: 64
    valid_lft: forever
    preferred_lft: forever
    flags:
    - permanent
  - address: "fe80::223:45ff:fe67:891a"
    prefix_len: 64
    valid_lft: forever
    preferred_lft: forever
    flags:
    - permanent"#;

const EXPECTED_IPV6_DYNAMIC_INFO: &str = r#"---
addresses:
  - address: "2001:db8:a::9"
    prefix_len: 64
    valid_lft: 116sec
    preferred_lft: 56sec
    flags: []
  - address: "fe80::223:45ff:fe67:891a"
    prefix_len: 64
    valid_lft: forever
    preferred_lft: forever
    flags:
    - permanent"#;

const EXPECTED_EMPTY_IPV6_INFO: &str = r#"---
addresses:
  - address: "fe80::223:45ff:fe67:891a"
    prefix_len: 64
    valid_lft: forever
    preferred_lft: forever"#;

#[test]
fn test_add_and_remove_ip() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(ADD_IP_CONF).unwrap();
        conf.apply().unwrap();
        wait_ipv6_dad();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_value_match(EXPECTED_IPV4_INFO, &iface.ipv4);
        assert_value_match(EXPECTED_IPV6_INFO, &iface.ipv6);
        let conf: NetConf = serde_yaml::from_str(DEL_IP_CONF).unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_eq!(iface.ipv4, None);
        assert_value_match(EXPECTED_EMPTY_IPV6_INFO, &iface.ipv6);
    });
}

#[test]
fn test_add_and_remove_dynamic_ip() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(ADD_IP_CONF_DYNAMIC).unwrap();
        conf.apply().unwrap();
        wait_ipv6_dad();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_value_match(EXPECTED_IPV4_DYNAMIC_INFO, &iface.ipv4);
        assert_value_match(EXPECTED_IPV6_DYNAMIC_INFO, &iface.ipv6);
        let conf: NetConf = serde_yaml::from_str(DEL_IP_CONF).unwrap();
        conf.apply().unwrap();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_eq!(iface.ipv4, None);
        assert_value_match(EXPECTED_EMPTY_IPV6_INFO, &iface.ipv6);
    });
}

#[test]
fn test_add_dynamic_ip_repeat() {
    with_veth_iface(|| {
        let conf: NetConf = serde_yaml::from_str(ADD_IP_CONF_DYNAMIC).unwrap();
        conf.apply().unwrap();
        conf.apply().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(2));
        conf.apply().unwrap();
        wait_ipv6_dad();
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME];
        let iface_type = &iface.iface_type;
        assert_eq!(iface_type, &crate::IfaceType::Veth);
        assert_value_match(EXPECTED_IPV4_DYNAMIC_INFO, &iface.ipv4);
        assert_value_match(EXPECTED_IPV6_DYNAMIC_INFO, &iface.ipv6);
    });
}

fn with_ipv6_token<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("ipv6token");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

#[test]
fn test_ipv6_token() {
    with_ipv6_token(|| {
        let state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get("eth1").unwrap();
        assert_eq!(
            iface
                .ipv6
                .as_ref()
                .and_then(|i| i.token.as_ref())
                .map(|i| i.to_string()),
            Some("::fac1".to_string())
        );
    })
}

fn with_ipv6_p2p<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("ipv6p2p");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

const EXPECTED_IPV6_P2P_INFO: &str = r#"---
addresses:
  - address: "2001:db8:f::1"
    prefix_len: 128
    valid_lft: forever
    preferred_lft: forever
    flags:
    - permanent
    peer: 2001:db8:f::2
    peer_prefix_len: 64
  - address: "fe80::223:45ff:fe67:891a"
    prefix_len: 64
    valid_lft: forever
    preferred_lft: forever
    flags:
    - permanent"#;

#[test]
fn test_ipv6_p2p() {
    with_ipv6_p2p(|| {
        wait_ipv6_dad();
        let state = NetState::retrieve().unwrap();
        let iface = state.ifaces.get("eth1").unwrap();
        assert_value_match(EXPECTED_IPV6_P2P_INFO, &iface.ipv6);
    })
}

fn wait_ipv6_dad() {
    std::thread::sleep(std::time::Duration::from_secs(5));
}
