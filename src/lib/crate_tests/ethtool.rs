// SPDX-License-Identifier: Apache-2.0

use crate::NetState;
use pretty_assertions::assert_eq;

use std::panic;

use super::utils::assert_value_match;

const IFACE_NAME0: &str = "sim0";
const IFACE_NAME1: &str = "sim1";

const EXPECTED_PAUSE_INFO: &str = r#"---
rx: true
tx: true
auto_negotiate: false"#;

const EXPECTED_FEATURE_INFO: &str = r#"---
fixed:
  esp-hw-offload: false
  esp-tx-csum-hw-offload: false
  fcoe-mtu: false
  highdma: true
  hsr-dup-offload: false
  hsr-fwd-offload: false
  hsr-tag-ins-offload: false
  hsr-tag-rm-offload: false
  hw-tc-offload: false
  l2-fwd-offload: false
  loopback: true
  macsec-hw-offload: false
  netns-local: true
  rx-all: false
  rx-checksum: true
  rx-fcs: false
  rx-gro-hw: false
  rx-hashing: false
  rx-lro: false
  rx-ntuple-filter: false
  rx-udp_tunnel-port-offload: false
  rx-vlan-filter: false
  rx-vlan-hw-parse: false
  rx-vlan-stag-filter: false
  rx-vlan-stag-hw-parse: false
  tls-hw-record: false
  tls-hw-rx-offload: false
  tls-hw-tx-offload: false
  tx-checksum-fcoe-crc: false
  tx-checksum-ip-generic: true
  tx-checksum-ipv4: false
  tx-checksum-ipv6: false
  tx-checksum-sctp: true
  tx-esp-segmentation: false
  tx-fcoe-segmentation: false
  tx-gre-csum-segmentation: false
  tx-gre-segmentation: false
  tx-gso-partial: false
  tx-gso-robust: false
  tx-ipxip4-segmentation: false
  tx-ipxip6-segmentation: false
  tx-lockless: true
  tx-nocache-copy: false
  tx-scatter-gather-fraglist: true
  tx-tunnel-remcsum-segmentation: false
  tx-udp_tnl-csum-segmentation: false
  tx-udp_tnl-segmentation: false
  tx-vlan-hw-insert: false
  tx-vlan-stag-hw-insert: false
  vlan-challenged: true
changeable:
  rx-gro: true
  rx-gro-list: false
  rx-udp-gro-forwarding: false
  tx-generic-segmentation: true
  tx-sctp-segmentation: true
  tx-tcp-ecn-segmentation: true
  tx-tcp-mangleid-segmentation: true
  tx-tcp-segmentation: true
  tx-tcp6-segmentation: true"#;

#[test]
fn test_get_ethtool_pause_yaml() {
    with_netdevsim_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface0 = &state.ifaces[IFACE_NAME0];
        let iface1 = &state.ifaces[IFACE_NAME1];
        assert_eq!(&iface0.iface_type, &crate::IfaceType::Ethernet);
        assert_eq!(&iface1.iface_type, &crate::IfaceType::Ethernet);
        assert_value_match(
            EXPECTED_PAUSE_INFO,
            &iface0.ethtool.as_ref().unwrap().pause,
        );
        assert_value_match(
            EXPECTED_PAUSE_INFO,
            &iface1.ethtool.as_ref().unwrap().pause,
        );
    });
}

#[test]
fn test_get_ethtool_feature_yaml_of_loopback() {
    let mut state = NetState::retrieve().unwrap();
    let iface = state.ifaces.get_mut("lo").unwrap();
    // These property value is different between Github CI and my Archlinux
    iface
        .ethtool
        .as_mut()
        .unwrap()
        .features
        .as_mut()
        .map(|features| features.fixed.remove("tx-gso-list"));
    iface
        .ethtool
        .as_mut()
        .unwrap()
        .features
        .as_mut()
        .map(|features| features.changeable.remove("tx-gso-list"));
    iface
        .ethtool
        .as_mut()
        .unwrap()
        .features
        .as_mut()
        .map(|features| features.fixed.remove("tx-udp-segmentation"));
    iface
        .ethtool
        .as_mut()
        .unwrap()
        .features
        .as_mut()
        .map(|features| features.changeable.remove("tx-udp-segmentation"));
    assert_eq!(&iface.iface_type, &crate::IfaceType::Loopback);
    assert_value_match(
        EXPECTED_FEATURE_INFO,
        &iface.ethtool.as_ref().unwrap().features,
    );
}

// TODO: There is no way to test the ethtool ring.

fn with_netdevsim_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("sim");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}

const IFACE_TUN_NAME: &str = "tun1";
const EXPECTED_ETHTOOL_COALESCE: &str = r#"---
rx_max_frames: 60"#;
const EXPECTED_ETHTOOL_LINK_MODE: &str = r#"---
auto_negotiate: false
ours: []
duplex: full"#;

#[test]
fn test_get_ethtool_coalesce_yaml() {
    with_tun_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_TUN_NAME];
        assert_value_match(
            EXPECTED_ETHTOOL_COALESCE,
            &iface.ethtool.as_ref().unwrap().coalesce,
        );
    });
}

#[test]
fn test_get_ethtool_link_mode_yaml() {
    with_tun_iface(|| {
        let state = NetState::retrieve().unwrap();
        let iface = &state.ifaces[IFACE_TUN_NAME];
        assert_value_match(
            EXPECTED_ETHTOOL_LINK_MODE,
            &iface.ethtool.as_ref().unwrap().link_mode,
        );
        assert!(
            iface
                .ethtool
                .as_ref()
                .unwrap()
                .link_mode
                .as_ref()
                .unwrap()
                .speed
                >= 10
        )
    });
}

fn with_tun_iface<T>(test: T)
where
    T: FnOnce() + panic::UnwindSafe,
{
    super::utils::set_network_environment("tun");

    let result = panic::catch_unwind(|| {
        test();
    });

    super::utils::clear_network_environment();
    assert!(result.is_ok())
}
