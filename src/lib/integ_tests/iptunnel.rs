// SPDX-License-Identifier: Apache-2.0

use std::panic;

use pretty_assertions::assert_eq;

use super::utils::assert_value_match;
use crate::NetState;

const IFACE_NAME_IPIP: &str = "ipip0";
const IFACE_NAME_SIT: &str = "sittest0";
const IFACE_NAME_IP6IP6: &str = "ip6ip60";
const IFACE_NAME_IPIP6: &str = "ipip60";

const EXPECTED_IPIP_INFO: &str = r#"---
name: ipip0
iface_type: ip_tunnel
ip_tunnel:
  local: 192.0.2.1
  remote: 192.0.2.2
  mode: ipip
  ttl: 42
  tos: 22"#;

const EXPECTED_SIT_INFO: &str = r#"---
name: sittest0
iface_type: ip_tunnel
ip_tunnel:
  local: 192.0.2.1
  remote: 192.0.2.2
  mode: sit
  ttl: 42
  tos: 22"#;

const EXPECTED_IP6IP6_INFO: &str = r#"---
name: ip6ip60
iface_type: ip_tunnel
ip_tunnel:
  local: 2001:db8:e::1
  remote: 2001:db8:e::ffff
  mode: ip6ip6
  ttl: 42
  ip6tun_flags:
    - cap_xmit
    - cap_rcv"#;

const EXPECTED_IPIP6_INFO: &str = r#"---
name: ipip60
iface_type: ip_tunnel
ip_tunnel:
  local: 2001:db8:f::1
  remote: 2001:db8:f::ffff
  mode: ipip6
  ttl: 42"#;

#[test]
fn test_get_ipip_iface_yaml() {
    with_iptunnel_iface("ipip", || {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME_IPIP];

        assert_eq!(iface.iface_type, crate::IfaceType::IpTunnel);

        assert_value_match(EXPECTED_IPIP_INFO, iface);
    });
}

#[test]
fn test_get_sit_iface_yaml() {
    with_iptunnel_iface("sit", || {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME_SIT];

        assert_eq!(iface.iface_type, crate::IfaceType::IpTunnel);
        assert_value_match(EXPECTED_SIT_INFO, iface);
    });
}

#[test]
fn test_get_ip6ip6_iface_yaml() {
    with_iptunnel_iface("ip6ip6", || {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME_IP6IP6];

        assert_eq!(iface.iface_type, crate::IfaceType::IpTunnel);
        assert_value_match(EXPECTED_IP6IP6_INFO, iface);
    });
}

#[test]
fn test_get_ipip6_iface_yaml() {
    with_iptunnel_iface("ipip6", || {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_NAME_IPIP6];

        assert_eq!(iface.iface_type, crate::IfaceType::IpTunnel);
        assert_value_match(EXPECTED_IPIP6_INFO, iface);
    });
}

fn with_iptunnel_iface<T>(iface_type: &str, test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment(iface_type);

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
