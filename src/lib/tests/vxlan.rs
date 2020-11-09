use nispor::NetState;
use pretty_assertions::assert_eq;
use serde_yaml;
use std::panic;

mod utils;

const IFACE_NAME: &str = "vxlan0";

const EXPECTED_IFACE_NAME: &str = r#"---
- name: vxlan0
  iface_type: vxlan
  state: unknown
  mtu: 1450
  flags:
    - broadcast
    - lower_up
    - multicast
    - running
    - up
  ipv6:
    addresses:
      - address: "fe80::223:45ff:fe67:891c"
        prefix_len: 64
        valid_lft: forever
        preferred_lft: forever
  mac_address: "00:23:45:67:89:1c"
  vxlan:
    remote: 8.8.8.8
    vxlan_id: 101
    base_iface: eth1
    local: 1.1.1.1
    ttl: 0
    tos: 0
    learning: true
    ageing: 300
    max_address: 0
    src_port_min: 0
    src_port_max: 0
    proxy: false
    rsc: false
    l2miss: false
    l3miss: false
    dst_port: 4789
    udp_check_sum: true
    udp6_zero_check_sum_tx: false
    udp6_zero_check_sum_rx: false
    remote_check_sum_tx: false
    remote_check_sum_rx: false
    gbp: false
    remote_check_sum_no_partial: false
    collect_metadata: false
    label: 0
    gpe: false
    ttl_inherit: false
    df: 0"#;

#[test]
fn test_get_vxlan_iface_yaml() {
    with_vxlan_iface(|| {
        if let Ok(state) = NetState::retrieve() {
            let iface = &state.ifaces[IFACE_NAME];
            assert_eq!(iface.iface_type, nispor::IfaceType::Vxlan);
            assert_eq!(
                serde_yaml::to_string(&vec![iface]).unwrap(),
                EXPECTED_IFACE_NAME
            );
        }
    });
}

fn with_vxlan_iface<T>(test: T) -> ()
where
    T: FnOnce() -> () + panic::UnwindSafe,
{
    utils::set_network_environment("vxlan");

    let result = panic::catch_unwind(|| {
        test();
    });

    utils::clear_network_environment();
    assert!(result.is_ok())
}
