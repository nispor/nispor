use crate::netlink::nla::parse_as_u16;
use crate::netlink::nla::parse_as_u32;
use crate::netlink::nla::parse_as_u8;
use crate::parse_as_mac;
use crate::BondMiiStatus;
use crate::BondSubordinateInfo;
use crate::BondSubordinateState;
use netlink_packet_route::rtnl::nlas::NlasIterator;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::net::Ipv4Addr;

// Using the kernel constant name.
const BOND_MODE_ROUNDROBIN: u8 = 0;
const BOND_MODE_ACTIVEBACKUP: u8 = 1;
const BOND_MODE_XOR: u8 = 2;
// const BOND_MODE_BROADCAST: u8 = 3;
const BOND_MODE_8023AD: u8 = 4;
const BOND_MODE_TLB: u8 = 5;
const BOND_MODE_ALB: u8 = 6;

// Using the sysfs mode name
const BOND_MODES: &[&str] = &[
    "balance-rr",
    "active-backup",
    "balance-xor",
    "broadcast",
    "802.3ad",
    "balance-tlb",
    "balance-alb",
];

fn bond_mode_u8_to_string(mode: u8) -> String {
    if let Some(mode_str) = BOND_MODES.get::<usize>(mode.into()) {
        mode_str.to_string()
    } else {
        format!("unknown: {}", mode)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
struct BondAdInfo {
    aggregator: u16,
    num_ports: u16,
    actor_key: u16,
    partner_key: u16,
    partner_mac: String,
}

const IFLA_BOND_MODE: u16 = 1;
const IFLA_BOND_AD_INFO: u16 = 23;

fn parse_as_nested_ipv4_addr(raw: &[u8]) -> Vec<Ipv4Addr> {
    let mut addresses = Vec::new();
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => {
                let data = nla.value();
                addresses
                    .push(Ipv4Addr::new(data[0], data[1], data[2], data[3]));
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    addresses
}

fn ipv4_addr_array_to_string(addrs: &[Ipv4Addr]) -> String {
    let mut rt = String::new();
    for i in 0..(addrs.len()) {
        rt.push_str(&addrs[i].to_string());
        if i != addrs.len() - 1 {
            rt.push_str(",");
        }
    }
    rt
}

fn parse_as_48_bits_mac(data: &[u8]) -> String {
    parse_as_mac(6, data)
}

const IFLA_BOND_AD_INFO_AGGREGATOR: u16 = 1;
const IFLA_BOND_AD_INFO_NUM_PORTS: u16 = 2;
const IFLA_BOND_AD_INFO_ACTOR_KEY: u16 = 3;
const IFLA_BOND_AD_INFO_PARTNER_KEY: u16 = 4;
const IFLA_BOND_AD_INFO_PARTNER_MAC: u16 = 5;

fn parse_ad_info(raw: &[u8]) -> BondAdInfo {
    let nlas = NlasIterator::new(raw);
    let mut ad_info = BondAdInfo::default();
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_AD_INFO_AGGREGATOR => {
                    ad_info.aggregator = parse_as_u16(nla.value())
                }
                IFLA_BOND_AD_INFO_NUM_PORTS => {
                    ad_info.num_ports = parse_as_u16(nla.value())
                }
                IFLA_BOND_AD_INFO_ACTOR_KEY => {
                    ad_info.actor_key = parse_as_u16(nla.value())
                }
                IFLA_BOND_AD_INFO_PARTNER_KEY => {
                    ad_info.partner_key = parse_as_u16(nla.value())
                }
                IFLA_BOND_AD_INFO_PARTNER_MAC => {
                    ad_info.partner_mac = parse_as_48_bits_mac(nla.value())
                }
                _ => {
                    eprintln!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    ad_info
}

fn get_bond_mode(raw: &[u8]) -> u8 {
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_MODE => {
                    return parse_as_u8(nla.value());
                }
                _ => {
                    eprintln!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    std::u8::MAX
}

// TODO: Use macro to generate function below
fn parse_active_subordinate(
    data: &[u8],
    mode: &u8,
) -> Option<(String, String)> {
    if [BOND_MODE_ACTIVEBACKUP, BOND_MODE_ALB, BOND_MODE_TLB].contains(mode) {
        Some((
            "active_subordinate".into(),
            format!("{}", parse_as_u32(data)),
        ))
    } else {
        None
    }
}

fn parse_miimon(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("miimon".into(), format!("{}", parse_as_u32(data))))
}

fn parse_void(_data: &[u8], _mode: &u8) -> Option<(String, String)> {
    None
}

fn parse_updelay(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("updelay".into(), format!("{}", parse_as_u32(data))))
}

fn parse_downdelay(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("downdelay".into(), format!("{}", parse_as_u32(data))))
}

fn parse_use_carrier(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("use_carrier".into(), format!("{}", parse_as_u8(data))))
}

fn parse_arp_interval(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("arp_interval".into(), format!("{}", parse_as_u32(data))))
}

fn parse_arp_ip_target(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some((
        "arp_ip_target".into(),
        format!(
            "{}",
            ipv4_addr_array_to_string(&parse_as_nested_ipv4_addr(data))
        ),
    ))
}

fn parse_arp_all_targets(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("arp_all_targets".into(), format!("{}", parse_as_u32(data))))
}

const ARP_VALIDATE_VALUES: &[&str] = &[
    "none",
    "active",
    "backup",
    "all",
    "filter",
    "filter_active",
    "filter_backup",
];

fn parse_arp_validate(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    let value_int = parse_as_u32(data);
    if let Ok(i) = usize::try_from(value_int) {
        if let Some(value) = ARP_VALIDATE_VALUES.get(i) {
            return Some(("arp_validate".into(), value.to_string()));
        }
    }
    Some(("arp_validate".into(), format!("unknown: {}", value_int)))
}

fn parse_primary(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if [BOND_MODE_ACTIVEBACKUP, BOND_MODE_ALB, BOND_MODE_TLB].contains(mode) {
        Some(("primary".into(), format!("{}", parse_as_u32(data))))
    } else {
        None
    }
}

const PRIMARY_RESELECT_VALUES: &[&str] = &["always", "better", "failure"];

fn parse_primary_reselect(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if [BOND_MODE_ACTIVEBACKUP, BOND_MODE_ALB, BOND_MODE_TLB].contains(mode) {
        let i: usize = parse_as_u8(data).into();
        if let Some(value) = PRIMARY_RESELECT_VALUES.get(i) {
            Some(("primary_reselect".into(), value.to_string()))
        } else {
            Some(("primary_reselect".into(), format!("unknown: {}", i)))
        }
    } else {
        None
    }
}

const FAIL_OVER_MAC_VALUES: &[&str] = &["none", "active", "follow"];

fn parse_fail_over_mac(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_ACTIVEBACKUP {
        let i: usize = parse_as_u8(data).into();
        if let Some(value) = FAIL_OVER_MAC_VALUES.get(i) {
            Some(("fail_over_mac".into(), value.to_string()))
        } else {
            Some(("fail_over_mac".into(), format!("unknown: {}", i)))
        }
    } else {
        None
    }
}

const XMIT_HASH_POLICY_VALUES: &[&str] =
    &["layer2", "layer2+3", "layer3+4", "encap2+3", "encap3+4"];

fn parse_xmit_hash_policy(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if [BOND_MODE_XOR, BOND_MODE_8023AD, BOND_MODE_TLB].contains(mode) {
        let i: usize = parse_as_u8(data).into();
        if let Some(value) = XMIT_HASH_POLICY_VALUES.get(i) {
            Some(("xmit_hash_policy".into(), value.to_string()))
        } else {
            Some(("xmit_hash_policy".into(), format!("unknown: {}", i)))
        }
    } else {
        None
    }
}

fn parse_resend_igmp(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if [
        BOND_MODE_ROUNDROBIN,
        BOND_MODE_ACTIVEBACKUP,
        BOND_MODE_TLB,
        BOND_MODE_ALB,
    ]
    .contains(mode)
    {
        Some(("resend_igmp".into(), format!("{}", parse_as_u32(data))))
    } else {
        None
    }
}

fn parse_num_peer_notif(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_ACTIVEBACKUP {
        Some(("num_peer_notif".into(), format!("{}", parse_as_u8(data))))
    } else {
        None
    }
}

const ALL_SUBORDINATES_ACTIVE_VALUES: &[&str] = &["dropped", "delivered"];

fn parse_all_subordinates_active(
    data: &[u8],
    _mode: &u8,
) -> Option<(String, String)> {
    let i: usize = parse_as_u8(data).into();
    if let Some(value) = ALL_SUBORDINATES_ACTIVE_VALUES.get(i) {
        Some(("all_subordinates_active".into(), value.to_string()))
    } else {
        Some(("all_subordinates_active".into(), format!("unknown: {}", i)))
    }
}

fn parse_min_links(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        Some(("min_links".into(), format!("{}", parse_as_u32(data))))
    } else {
        None
    }
}

fn parse_lp_interval(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if [BOND_MODE_TLB, BOND_MODE_ALB].contains(mode) {
        Some(("lp_interval".into(), format!("{}", parse_as_u32(data))))
    } else {
        None
    }
}

fn parse_packets_per_subordinate(
    data: &[u8],
    mode: &u8,
) -> Option<(String, String)> {
    if *mode == BOND_MODE_ROUNDROBIN {
        Some((
            "packets_per_subordinate".into(),
            format!("{}", parse_as_u32(data)),
        ))
    } else {
        None
    }
}

const LACP_RATE_VALUES: &[&str] = &["slow", "fast"];

fn parse_ad_lacp_rate(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        let i: usize = parse_as_u8(data).into();
        if let Some(value) = LACP_RATE_VALUES.get(i) {
            Some(("lacp_rate".into(), value.to_string()))
        } else {
            Some(("lacp_rate".into(), format!("unknown: {}", i)))
        }
    } else {
        None
    }
}

const AD_SELECT_VALUES: &[&str] = &["stable", "bandwidth", "count"];

fn parse_ad_select(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        let i: usize = parse_as_u8(data).into();
        if let Some(value) = AD_SELECT_VALUES.get(i) {
            Some(("ad_select".into(), value.to_string()))
        } else {
            Some(("ad_select".into(), format!("unknown: {}", i)))
        }
    } else {
        None
    }
}

fn parse_ad_actor_sys_prio(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        Some((
            "ad_actor_sys_prio".into(),
            format!("{}", parse_as_u16(data)),
        ))
    } else {
        None
    }
}

fn parse_ad_user_port_key(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        Some(("ad_user_port_key".into(), format!("{}", parse_as_u16(data))))
    } else {
        None
    }
}

fn parse_ad_actor_system(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_8023AD {
        Some(("ad_actor_system".into(), parse_as_48_bits_mac(data)))
    } else {
        None
    }
}

fn parse_tlb_dynamic_lb(data: &[u8], mode: &u8) -> Option<(String, String)> {
    if *mode == BOND_MODE_TLB {
        Some(("tlb_dynamic_lb".into(), format!("{}", parse_as_u8(data))))
    } else {
        None
    }
}

fn parse_peer_notif_delay(data: &[u8], _mode: &u8) -> Option<(String, String)> {
    Some(("peer_notif_delay".into(), format!("{}", parse_as_u32(data))))
}

const NLA_PARSE_FUNS: &[fn(&[u8], &u8) -> Option<(String, String)>] = &[
    parse_void, // IFLA_BOND_UNSPEC
    parse_void, // IFLA_BOND_MODE
    parse_active_subordinate,
    parse_miimon,
    parse_updelay,
    parse_downdelay,
    parse_use_carrier,
    parse_arp_interval,
    parse_arp_ip_target,
    parse_arp_validate,
    parse_arp_all_targets,
    parse_primary,
    parse_primary_reselect,
    parse_fail_over_mac,
    parse_xmit_hash_policy,
    parse_resend_igmp,
    parse_num_peer_notif,
    parse_all_subordinates_active,
    parse_min_links,
    parse_lp_interval,
    parse_packets_per_subordinate,
    parse_ad_lacp_rate,
    parse_ad_select,
    parse_void, // IFLA_BOND_AD_INFO, handled by parse_ad_info().
    parse_ad_actor_sys_prio,
    parse_ad_user_port_key,
    parse_ad_actor_system,
    parse_tlb_dynamic_lb,
    parse_peer_notif_delay,
];

pub(crate) fn parse_bond_info(raw: &[u8]) -> HashMap<String, String> {
    let mut bond_options: HashMap<String, String> = HashMap::new();
    let mode = get_bond_mode(raw);
    let nlas = NlasIterator::new(raw);
    for nla in nlas {
        match nla {
            Ok(nla) => {
                if let Some(func) =
                    NLA_PARSE_FUNS.get::<usize>(nla.kind().into())
                {
                    if let Some((name, value)) = func(nla.value(), &mode) {
                        bond_options.insert(name, value);
                    }
                } else if nla.kind() == IFLA_BOND_AD_INFO {
                    let ad_info = parse_ad_info(nla.value());
                    bond_options.insert(
                        "ad_aggregator".into(),
                        format!("{}", ad_info.aggregator),
                    );
                    bond_options.insert(
                        "ad_num_ports".into(),
                        format!("{}", ad_info.num_ports),
                    );
                    bond_options.insert(
                        "ad_actor_key".into(),
                        format!("{}", ad_info.actor_key),
                    );
                    bond_options.insert(
                        "ad_partner_key".into(),
                        format!("{}", ad_info.partner_key),
                    );
                    bond_options.insert(
                        "ad_partner_mac".into(),
                        format!("{}", &ad_info.partner_mac),
                    );
                } else {
                    bond_options.insert(
                        format!("{}", nla.kind()),
                        format!("{:?}", nla.value()),
                    );
                }
            }
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }

    bond_options.insert("mode".to_string(), bond_mode_u8_to_string(mode));

    if let Some(value) = bond_options.get("num_peer_notif") {
        let value1 = value.clone();
        let value2 = value.clone();
        bond_options.insert("num_unsol_na".into(), value1);
        bond_options.insert("num_grat_arp".into(), value2);
    }
    bond_options.remove("num_peer_notif");
    bond_options
}

const IFLA_BOND_SLAVE_STATE: u16 = 1;
const IFLA_BOND_SLAVE_MII_STATUS: u16 = 2;
const IFLA_BOND_SLAVE_LINK_FAILURE_COUNT: u16 = 3;
const IFLA_BOND_SLAVE_PERM_HWADDR: u16 = 4;
const IFLA_BOND_SLAVE_QUEUE_ID: u16 = 5;
const IFLA_BOND_SLAVE_AD_AGGREGATOR_ID: u16 = 6;
const IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE: u16 = 7;
const IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE: u16 = 8;

pub(crate) fn parse_bond_subordinate_info(raw: &[u8]) -> BondSubordinateInfo {
    let nlas = NlasIterator::new(raw);
    let mut subordinate_state = BondSubordinateState::Unknown;
    let mut mii_status = BondMiiStatus::Unknown;
    let mut link_failure_count = std::u32::MAX;
    let mut perm_hwaddr = String::new();
    let mut queue_id = std::u16::MAX;
    let mut ad_aggregator_id = None;
    let mut ad_actor_oper_port_state = None;
    let mut ad_partner_oper_port_state = None;
    for nla in nlas {
        match nla {
            Ok(nla) => match nla.kind() {
                IFLA_BOND_SLAVE_STATE => {
                    subordinate_state = parse_as_u8(nla.value()).into()
                }
                IFLA_BOND_SLAVE_MII_STATUS => {
                    mii_status = parse_as_u8(nla.value()).into()
                }
                IFLA_BOND_SLAVE_LINK_FAILURE_COUNT => {
                    link_failure_count = parse_as_u32(nla.value())
                }
                IFLA_BOND_SLAVE_PERM_HWADDR => {
                    perm_hwaddr = parse_as_mac(nla.value_length(), nla.value());
                }
                IFLA_BOND_SLAVE_QUEUE_ID => {
                    queue_id = parse_as_u16(nla.value())
                }
                IFLA_BOND_SLAVE_AD_AGGREGATOR_ID => {
                    ad_aggregator_id = Some(parse_as_u16(nla.value()));
                }
                IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE => {
                    ad_actor_oper_port_state = Some(parse_as_u8(nla.value()));
                }
                IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE => {
                    ad_partner_oper_port_state =
                        Some(parse_as_u16(nla.value()));
                }
                _ => {
                    eprintln!(
                        "unknown nla kind {} value: {:?}",
                        nla.kind(),
                        nla.value()
                    );
                }
            },
            Err(e) => {
                eprintln!("{}", e);
            }
        }
    }
    BondSubordinateInfo {
        subordinate_state,
        mii_status,
        link_failure_count,
        perm_hwaddr,
        queue_id,
        ad_aggregator_id,
        ad_actor_oper_port_state,
        ad_partner_oper_port_state,
    }
}
