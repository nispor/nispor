use core::fmt::Display;
use core::fmt::Error;
use core::fmt::Formatter;
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::transmute;
use std::net::Ipv4Addr;
use std::slice;

// TODO: Create a macro for these u8 enums.

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
pub enum BondMode {
    BalanceRR,
    ActiveBackup,
    BalanceXOR,
    Broadcast,
    IEEE8023AD,
    BalanceTLB,
    BalanceALB,
    Unknown = std::u8::MAX,
}

const _LAST_BOND_MODE: BondMode = BondMode::BalanceALB;

impl Display for BondMode {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondMode::BalanceRR => write!(f, "balance-rr"),
            BondMode::ActiveBackup => write!(f, "active-backup"),
            BondMode::BalanceXOR => write!(f, "balance-xor"),
            BondMode::Broadcast => write!(f, "broadcast"),
            BondMode::IEEE8023AD => write!(f, "802.3ad"),
            BondMode::BalanceTLB => write!(f, "balance-tlb"),
            BondMode::BalanceALB => write!(f, "balance-alb"),
            BondMode::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<u8> for BondMode {
    fn from(d: u8) -> Self {
        if d <= _LAST_BOND_MODE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondMode::Unknown
        }
    }
}

impl From<&str> for BondMode {
    fn from(s: &str) -> Self {
        match s {
            "balance-rr" => BondMode::BalanceRR,
            "active-backup" => BondMode::ActiveBackup,
            "balance-xor" => BondMode::BalanceXOR,
            "broadcast" => BondMode::Broadcast,
            "802.3ad" => BondMode::IEEE8023AD,
            "balance-tlb" => BondMode::BalanceTLB,
            "balance-alb" => BondMode::BalanceALB,
            _ => BondMode::Unknown,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondPrimaryReselect {
    Always,
    Better,
    Failure,
    Unknown = std::u8::MAX,
}

const _LAST_PR: BondPrimaryReselect = BondPrimaryReselect::Failure;

impl From<u8> for BondPrimaryReselect {
    fn from(d: u8) -> Self {
        if d <= _LAST_PR as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondPrimaryReselect::Unknown
        }
    }
}

impl Display for BondPrimaryReselect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondPrimaryReselect::Always => write!(f, "always"),
            BondPrimaryReselect::Better => write!(f, "better"),
            BondPrimaryReselect::Failure => write!(f, "failure"),
            BondPrimaryReselect::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondFailOverMac {
    None,
    Active,
    Follow,
    Unknown = std::u8::MAX,
}

const _LAST_FOM: BondFailOverMac = BondFailOverMac::Follow;

impl From<u8> for BondFailOverMac {
    fn from(d: u8) -> Self {
        if d <= _LAST_FOM as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondFailOverMac::Unknown
        }
    }
}

impl Display for BondFailOverMac {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondFailOverMac::None => write!(f, "none"),
            BondFailOverMac::Active => write!(f, "active"),
            BondFailOverMac::Follow => write!(f, "follow"),
            BondFailOverMac::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondXmitHashPolicy {
    Layer2,
    Layer2_3,
    Layer3_4,
    Encap2_3,
    Encap3_4,
    Unknown = std::u8::MAX,
}

const _LAST_XHP: BondXmitHashPolicy = BondXmitHashPolicy::Encap3_4;

impl From<u8> for BondXmitHashPolicy {
    fn from(d: u8) -> Self {
        if d <= _LAST_XHP as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondXmitHashPolicy::Unknown
        }
    }
}

impl Display for BondXmitHashPolicy {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondXmitHashPolicy::Layer2 => write!(f, "layer2"),
            BondXmitHashPolicy::Layer2_3 => write!(f, "layer2+3"),
            BondXmitHashPolicy::Layer3_4 => write!(f, "layer3+4"),
            BondXmitHashPolicy::Encap2_3 => write!(f, "encap2+3"),
            BondXmitHashPolicy::Encap3_4 => write!(f, "encap3+4"),
            BondXmitHashPolicy::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondAllSlavesActive {
    Dropped,
    Delivered,
    Unknown = std::u8::MAX,
}

const _LAST_ASA: BondAllSlavesActive = BondAllSlavesActive::Delivered;

impl From<u8> for BondAllSlavesActive {
    fn from(d: u8) -> Self {
        if d <= _LAST_ASA as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondAllSlavesActive::Unknown
        }
    }
}

impl Display for BondAllSlavesActive {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondAllSlavesActive::Dropped => write!(f, "dropped"),
            BondAllSlavesActive::Delivered => write!(f, "delivered"),
            BondAllSlavesActive::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondAdLacpRate {
    Slow,
    Fast,
    Unknown = std::u8::MAX,
}

const _LAST_AD_LACP_RATE: BondAdLacpRate = BondAdLacpRate::Fast;

impl From<u8> for BondAdLacpRate {
    fn from(d: u8) -> Self {
        if d <= _LAST_AD_LACP_RATE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondAdLacpRate::Unknown
        }
    }
}

impl Display for BondAdLacpRate {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondAdLacpRate::Slow => write!(f, "slow"),
            BondAdLacpRate::Fast => write!(f, "fast"),
            BondAdLacpRate::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondAdSelect {
    Stable,
    Bandwidth,
    Count,
    Unknown = std::u8::MAX,
}

const _LAST_AD_SELECT: BondAdSelect = BondAdSelect::Count;

impl From<u8> for BondAdSelect {
    fn from(d: u8) -> Self {
        if d <= _LAST_AD_SELECT as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondAdSelect::Unknown
        }
    }
}

impl Display for BondAdSelect {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondAdSelect::Stable => write!(f, "stable"),
            BondAdSelect::Bandwidth => write!(f, "bandwidth"),
            BondAdSelect::Count => write!(f, "count"),
            BondAdSelect::Unknown => write!(f, "unknown"),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, Clone)]
pub enum BondTlbDynamicLb {
    Enabled,
    Disabled,
    Unknown = std::u8::MAX,
}

const _LAST_TLB_DYNAMIC_LB: BondTlbDynamicLb = BondTlbDynamicLb::Disabled;

impl From<u8> for BondTlbDynamicLb {
    fn from(d: u8) -> Self {
        if d <= _LAST_TLB_DYNAMIC_LB as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondTlbDynamicLb::Unknown
        }
    }
}

impl Display for BondTlbDynamicLb {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            BondTlbDynamicLb::Enabled => write!(f, "enabled"),
            BondTlbDynamicLb::Disabled => write!(f, "disabled"),
            BondTlbDynamicLb::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct BondAdInfo {
    pub aggregator: u16,
    pub num_ports: u16,
    pub actor_key: u16,
    pub partner_key: u16,
    pub partner_mac: String,
}

const IFLA_BOND_MODE: u16 = 1;
const IFLA_BOND_ACTIVE_SLAVE: u16 = 2;
const IFLA_BOND_MIIMON: u16 = 3;
const IFLA_BOND_UPDELAY: u16 = 4;
const IFLA_BOND_DOWNDELAY: u16 = 5;
const IFLA_BOND_USE_CARRIER: u16 = 6;
const IFLA_BOND_ARP_INTERVAL: u16 = 7;
const IFLA_BOND_ARP_IP_TARGET: u16 = 8;
const IFLA_BOND_ARP_VALIDATE: u16 = 9;
const IFLA_BOND_ARP_ALL_TARGETS: u16 = 10;
const IFLA_BOND_PRIMARY: u16 = 11;
const IFLA_BOND_PRIMARY_RESELECT: u16 = 12;
const IFLA_BOND_FAIL_OVER_MAC: u16 = 13;
const IFLA_BOND_XMIT_HASH_POLICY: u16 = 14;
const IFLA_BOND_RESEND_IGMP: u16 = 15;
const IFLA_BOND_NUM_PEER_NOTIF: u16 = 16;
const IFLA_BOND_ALL_SLAVES_ACTIVE: u16 = 17;
const IFLA_BOND_MIN_LINKS: u16 = 18;
const IFLA_BOND_LP_INTERVAL: u16 = 19;
const IFLA_BOND_PACKETS_PER_SLAVE: u16 = 20;
const IFLA_BOND_AD_LACP_RATE: u16 = 21;
const IFLA_BOND_AD_SELECT: u16 = 22;
const IFLA_BOND_AD_INFO: u16 = 23;
const IFLA_BOND_AD_ACTOR_SYS_PRIO: u16 = 24;
const IFLA_BOND_AD_USER_PORT_KEY: u16 = 25;
const IFLA_BOND_AD_ACTOR_SYSTEM: u16 = 26;
const IFLA_BOND_TLB_DYNAMIC_LB: u16 = 27;
const IFLA_BOND_PEER_NOTIF_DELAY: u16 = 28;

const NL_ATTR_HDR_LEN: usize = 4;

#[derive(Debug, Eq, PartialEq, Clone)]
struct NetLinkAttrHeader {
    data_len: usize,
    nla_len: usize,
    nla_type: u16,
}

const NLA_ALIGNTO: usize = 4;

fn parse_nla_header(data: *const u8) -> NetLinkAttrHeader {
    let mut data_len: usize = unsafe {
        transmute::<[u8; 2], u16>([*data, *(data.wrapping_offset(1))])
    }
    .into();
    let nla_type: u16 = unsafe {
        transmute::<[u8; 2], u16>([
            *(data.wrapping_offset(2)),
            *(data.wrapping_offset(3)),
        ])
    };

    // Align nla_len by NLA_ALIGNTO
    let nla_len = ((data_len + NLA_ALIGNTO - 1) / NLA_ALIGNTO) * NLA_ALIGNTO;
    data_len = data_len - NL_ATTR_HDR_LEN;
    NetLinkAttrHeader {
        data_len,
        nla_len,
        nla_type,
    }
}

fn parse_as_u8(data: &[u8]) -> u8 {
    data[0]
}

fn parse_as_u16(data: &[u8]) -> u16 {
    unsafe { transmute::<[u8; 2], u16>([data[0], data[1]]) }
}

fn parse_as_u32(data: &[u8]) -> u32 {
    unsafe { transmute::<[u8; 4], u32>([data[0], data[1], data[2], data[3]]) }
}

fn parse_as_nested_ipv4_addr(data: &[u8]) -> Vec<Ipv4Addr> {
    let mut i: usize = 0;
    let mut addrs = Vec::new();
    while i < data.len() {
        let hdr_ptr = data.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = data
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        addrs.push(Ipv4Addr::new(data[0], data[1], data[2], data[3]));
        i = i + hdr.nla_len;
    }
    addrs
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

fn parse_as_mac(mac_len: usize, data: &[u8]) -> String {
    let mut rt = String::new();
    for i in 0..mac_len {
        rt.push_str(&format!("{:X}", data[i]));
        if i != mac_len - 1 {
            rt.push_str(":");
        }
    }
    rt
}

const IFLA_BOND_AD_INFO_AGGREGATOR: u16 = 1;
const IFLA_BOND_AD_INFO_NUM_PORTS: u16 = 2;
const IFLA_BOND_AD_INFO_ACTOR_KEY: u16 = 3;
const IFLA_BOND_AD_INFO_PARTNER_KEY: u16 = 4;
const IFLA_BOND_AD_INFO_PARTNER_MAC: u16 = 5;

fn parse_ad_info(data: &[u8]) -> BondAdInfo {
    let mut i: usize = 0;
    let mut ad_info = BondAdInfo::default();
    while i < data.len() {
        let hdr_ptr = data.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = data
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        match hdr.nla_type {
            IFLA_BOND_AD_INFO_AGGREGATOR => {
                ad_info.aggregator = parse_as_u16(data)
            }
            IFLA_BOND_AD_INFO_NUM_PORTS => {
                ad_info.num_ports = parse_as_u16(data)
            }
            IFLA_BOND_AD_INFO_ACTOR_KEY => {
                ad_info.actor_key = parse_as_u16(data)
            }
            IFLA_BOND_AD_INFO_PARTNER_KEY => {
                ad_info.partner_key = parse_as_u16(data)
            }
            IFLA_BOND_AD_INFO_PARTNER_MAC => {
                ad_info.partner_mac = parse_as_48_bits_mac(data)
            }
            _ => (),
        }
        i = i + hdr.nla_len;
    }
    ad_info
}

pub(crate) fn parse_bond_info(raw: &[u8]) -> HashMap<String, String> {
    let mut i: usize = 0;
    let mut bond_options: HashMap<String, String> = HashMap::new();

    // TODO: Convert this into a iterator or better dedup way
    while i < raw.len() {
        let hdr_ptr = raw.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = raw
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        match hdr.nla_type {
            IFLA_BOND_MODE => bond_options.insert(
                "mode".into(),
                format!("{}", BondMode::from(parse_as_u8(data))),
            ),
            IFLA_BOND_ACTIVE_SLAVE => bond_options.insert(
                "active_slave".into(),
                format!("{}", parse_as_u32(data)),
            ), // TODO: Convert kernel link interface to interface name
            IFLA_BOND_MIIMON => bond_options
                .insert("miimon".into(), format!("{}", parse_as_u32(data))),
            IFLA_BOND_UPDELAY => bond_options
                .insert("updelay".into(), format!("{}", parse_as_u32(data))),
            IFLA_BOND_DOWNDELAY => bond_options
                .insert("downdelay".into(), format!("{}", parse_as_u32(data))),
            IFLA_BOND_USE_CARRIER => bond_options
                .insert("use_carrier".into(), format!("{}", parse_as_u8(data))),
            IFLA_BOND_ARP_INTERVAL => bond_options.insert(
                "arp_interval".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_ARP_IP_TARGET => bond_options.insert(
                "arp_ip_target".into(),
                format!(
                    "{}",
                    ipv4_addr_array_to_string(&parse_as_nested_ipv4_addr(data))
                ),
            ),
            IFLA_BOND_ARP_VALIDATE => bond_options.insert(
                "arp_validate".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_ARP_ALL_TARGETS => bond_options.insert(
                "arp_all_targets".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_PRIMARY => bond_options
                .insert("primary".into(), format!("{}", parse_as_u32(data))),
            IFLA_BOND_PRIMARY_RESELECT => bond_options.insert(
                "primary_reselect".into(),
                format!("{}", BondPrimaryReselect::from(parse_as_u8(data))),
            ),
            IFLA_BOND_FAIL_OVER_MAC => bond_options.insert(
                "fail_over_mac".into(),
                format!("{}", BondFailOverMac::from(parse_as_u8(data))),
            ),
            IFLA_BOND_XMIT_HASH_POLICY => bond_options.insert(
                "xmit_hash_policy".into(),
                format!("{}", BondXmitHashPolicy::from(parse_as_u8(data))),
            ),
            IFLA_BOND_RESEND_IGMP => bond_options.insert(
                "resend_igmp".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_NUM_PEER_NOTIF => {
                let num_peer_notify = parse_as_u32(data);
                bond_options.insert(
                    "num_unsol_na".into(),
                    format!("{}", num_peer_notify),
                );
                bond_options.insert(
                    "num_grat_arp".into(),
                    format!("{}", num_peer_notify),
                )
            }
            IFLA_BOND_ALL_SLAVES_ACTIVE => bond_options.insert(
                "all_slaves_active".into(),
                format!("{}", BondAllSlavesActive::from(parse_as_u8(data))),
            ),
            IFLA_BOND_MIN_LINKS => bond_options
                .insert("min_links".into(), format!("{}", parse_as_u32(data))),
            IFLA_BOND_LP_INTERVAL => bond_options.insert(
                "lp_interval".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_PACKETS_PER_SLAVE => bond_options.insert(
                "packets_per_slave".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_AD_LACP_RATE => bond_options.insert(
                "ad_lacp_rate".into(),
                format!("{}", BondAdLacpRate::from(parse_as_u8(data))),
            ),
            IFLA_BOND_AD_SELECT => bond_options.insert(
                "ad_select".into(),
                format!("{}", BondAdSelect::from(parse_as_u8(data))),
            ),
            IFLA_BOND_AD_ACTOR_SYS_PRIO => bond_options.insert(
                "ad_actor_sys_prio".into(),
                format!("{}", parse_as_u16(data)),
            ),
            IFLA_BOND_AD_USER_PORT_KEY => bond_options.insert(
                "ad_user_port_key".into(),
                format!("{}", parse_as_u16(data)),
            ),
            IFLA_BOND_AD_ACTOR_SYSTEM => bond_options.insert(
                "ad_actor_system".into(),
                format!("{}", parse_as_48_bits_mac(data)),
            ),
            IFLA_BOND_TLB_DYNAMIC_LB => bond_options.insert(
                "tlb_dynamic_lb".into(),
                format!("{}", BondTlbDynamicLb::from(parse_as_u8(data))),
            ),
            IFLA_BOND_PEER_NOTIF_DELAY => bond_options.insert(
                "peer_notif_delay".into(),
                format!("{}", parse_as_u32(data)),
            ),
            IFLA_BOND_AD_INFO => {
                let ad_info = parse_ad_info(data);
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
                )
            }
            _ => bond_options
                .insert(format!("{}", hdr.nla_type), format!("{:?}", data)),
        };
        i = i + hdr.nla_len;
    }

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

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondSlaveState {
    Active,
    Backup,
    Unknown = std::u8::MAX,
}

const _LAST_BOND_SLAVE_STATE: BondSlaveState = BondSlaveState::Backup;

impl From<u8> for BondSlaveState {
    fn from(d: u8) -> Self {
        if d <= _LAST_BOND_SLAVE_STATE as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondSlaveState::Unknown
        }
    }
}

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum BondMiiStatus {
    LinkUp,
    LinkFail,
    LinkDown,
    LinkBack,
    Unknown = std::u8::MAX,
}

const _LAST_MII_STATUS: BondMiiStatus = BondMiiStatus::LinkBack;

impl From<u8> for BondMiiStatus {
    fn from(d: u8) -> Self {
        if d <= _LAST_MII_STATUS as u8 {
            unsafe { transmute(d as u8) }
        } else {
            BondMiiStatus::Unknown
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct BondSlaveInfo {
    pub slave_state: BondSlaveState,
    pub mii_status: BondMiiStatus,
    pub link_failure_count: u32,
    pub perm_hwaddr: String,
    pub queue_id: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_aggregator_id: Option<u16>,
    // 802.3ad port state definitions (43.4.2.2 in the 802.3ad standard)
    // bit map of LACP_STATE_XXX
    // TODO: Find a rust way of showing it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_actor_oper_port_state: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_partner_oper_port_state: Option<u16>,
}

pub(crate) fn parse_bond_slave_info(raw: &[u8]) -> BondSlaveInfo {
    let mut i: usize = 0;

    let mut slave_state = BondSlaveState::Unknown;
    let mut mii_status = BondMiiStatus::Unknown;
    let mut link_failure_count = std::u32::MAX;
    let mut perm_hwaddr = String::new();
    let mut queue_id = std::u16::MAX;
    let mut ad_aggregator_id = None;
    let mut ad_actor_oper_port_state = None;
    let mut ad_partner_oper_port_state = None;

    while i < raw.len() {
        let hdr_ptr = raw.as_ptr().wrapping_offset(i.try_into().unwrap());
        let hdr = parse_nla_header(hdr_ptr);
        let data_ptr = raw
            .as_ptr()
            .wrapping_offset((i + NL_ATTR_HDR_LEN).try_into().unwrap());
        let data = unsafe {
            slice::from_raw_parts(data_ptr, hdr.nla_len - NL_ATTR_HDR_LEN)
        };
        match hdr.nla_type {
            IFLA_BOND_SLAVE_STATE => slave_state = parse_as_u8(data).into(),
            IFLA_BOND_SLAVE_MII_STATUS => mii_status = parse_as_u8(data).into(),
            IFLA_BOND_SLAVE_LINK_FAILURE_COUNT => {
                link_failure_count = parse_as_u32(data)
            }
            IFLA_BOND_SLAVE_PERM_HWADDR => {
                perm_hwaddr = parse_as_mac(hdr.data_len, data);
            }
            IFLA_BOND_SLAVE_QUEUE_ID => queue_id = parse_as_u16(data),
            IFLA_BOND_SLAVE_AD_AGGREGATOR_ID => {
                ad_aggregator_id = Some(parse_as_u16(data));
            }
            IFLA_BOND_SLAVE_AD_ACTOR_OPER_PORT_STATE => {
                ad_actor_oper_port_state = Some(parse_as_u8(data));
            }
            IFLA_BOND_SLAVE_AD_PARTNER_OPER_PORT_STATE => {
                ad_partner_oper_port_state = Some(parse_as_u16(data));
            }
            _ => {
                eprintln!("unknown nla_type {} data: {:?}", hdr.nla_type, data);
            }
        }
        i = i + hdr.nla_len;
    }

    BondSlaveInfo {
        slave_state,
        mii_status,
        link_failure_count,
        perm_hwaddr,
        queue_id,
        ad_aggregator_id,
        ad_actor_oper_port_state,
        ad_partner_oper_port_state,
    }
}
