// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr};

use netlink_packet_route::link::nlas::{
    self, InfoBond, InfoBondPort, InfoData,
};
use serde::{Deserialize, Serialize};

use super::super::mac::parse_as_mac;
use crate::{ControllerType, Iface, IfaceType, NisporError};

const BOND_MODE_ROUNDROBIN: u8 = 0;
const BOND_MODE_ACTIVEBACKUP: u8 = 1;
const BOND_MODE_XOR: u8 = 2;
const BOND_MODE_BROADCAST: u8 = 3;
const BOND_MODE_8023AD: u8 = 4;
const BOND_MODE_TLB: u8 = 5;
const BOND_MODE_ALB: u8 = 6;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BondMode {
    #[serde(rename = "balance-rr")]
    BalanceRoundRobin,
    #[serde(rename = "active-backup")]
    ActiveBackup,
    #[serde(rename = "balance-xor")]
    BalanceXor,
    #[serde(rename = "broadcast")]
    Broadcast,
    #[serde(rename = "802.3ad")]
    Ieee8021AD,
    #[serde(rename = "balance-tlb")]
    BalanceTlb,
    #[serde(rename = "balance-alb")]
    BalanceAlb,
    Other(u8),
    Unknown,
}

impl Default for BondMode {
    fn default() -> Self {
        Self::BalanceRoundRobin
    }
}

impl From<u8> for BondMode {
    fn from(d: u8) -> Self {
        match d {
            BOND_MODE_ROUNDROBIN => Self::BalanceRoundRobin,
            BOND_MODE_ACTIVEBACKUP => Self::ActiveBackup,
            BOND_MODE_XOR => Self::BalanceXor,
            BOND_MODE_BROADCAST => Self::Broadcast,
            BOND_MODE_8023AD => Self::Ieee8021AD,
            BOND_MODE_TLB => Self::BalanceTlb,
            BOND_MODE_ALB => Self::BalanceAlb,
            _ => Self::Other(d),
        }
    }
}

impl From<BondMode> for u8 {
    fn from(v: BondMode) -> u8 {
        match v {
            BondMode::BalanceRoundRobin => BOND_MODE_ROUNDROBIN,
            BondMode::ActiveBackup => BOND_MODE_ACTIVEBACKUP,
            BondMode::BalanceXor => BOND_MODE_XOR,
            BondMode::Broadcast => BOND_MODE_BROADCAST,
            BondMode::Ieee8021AD => BOND_MODE_8023AD,
            BondMode::BalanceTlb => BOND_MODE_TLB,
            BondMode::BalanceAlb => BOND_MODE_ALB,
            BondMode::Other(d) => d,
            BondMode::Unknown => {
                log::warn!(
                    "Treating BondMode::Unknown as \
                    BondMode::BalanceRoundRobin"
                );
                BOND_MODE_ROUNDROBIN
            }
        }
    }
}

impl std::fmt::Display for BondMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BalanceRoundRobin => write!(f, "balance-rr"),
            Self::ActiveBackup => write!(f, "active-backup"),
            Self::BalanceXor => write!(f, "balance-xor"),
            Self::Broadcast => write!(f, "broadcast"),
            Self::Ieee8021AD => write!(f, "802.3ad"),
            Self::BalanceTlb => write!(f, "balance-tlb"),
            Self::BalanceAlb => write!(f, "balance-alb"),
            Self::Other(u) => write!(f, "{u}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondModeArpAllTargets {
    Any,
    All,
    Other(u32),
}

const BOND_OPT_ARP_ALL_TARGETS_ANY: u32 = 0;
const BOND_OPT_ARP_ALL_TARGETS_ALL: u32 = 1;

impl From<u32> for BondModeArpAllTargets {
    fn from(d: u32) -> Self {
        match d {
            BOND_OPT_ARP_ALL_TARGETS_ANY => Self::Any,
            BOND_OPT_ARP_ALL_TARGETS_ALL => Self::All,
            _ => Self::Other(d),
        }
    }
}

const BOND_ARP_VALIDATE_NONE: u32 = 0;
const BOND_ARP_VALIDATE_ACTIVE: u32 = 1 << BOND_STATE_ACTIVE as u32;
const BOND_ARP_VALIDATE_BACKUP: u32 = 1 << BOND_STATE_BACKUP as u32;
const BOND_ARP_VALIDATE_ALL: u32 =
    BOND_ARP_VALIDATE_ACTIVE | BOND_ARP_VALIDATE_BACKUP;
const BOND_ARP_FILTER: u32 = BOND_ARP_VALIDATE_ALL + 1;
const BOND_ARP_FILTER_ACTIVE: u32 = BOND_ARP_VALIDATE_ACTIVE | BOND_ARP_FILTER;
const BOND_ARP_FILTER_BACKUP: u32 = BOND_ARP_VALIDATE_BACKUP | BOND_ARP_FILTER;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondArpValidate {
    None,
    Active,
    Backup,
    All,
    Filter,
    #[serde(rename = "filter_active")]
    FilterActive,
    #[serde(rename = "filter_backkup")]
    FilterBackup,
    Other(u32),
}

impl From<u32> for BondArpValidate {
    fn from(d: u32) -> Self {
        match d {
            BOND_ARP_VALIDATE_NONE => Self::None,
            BOND_ARP_VALIDATE_ACTIVE => Self::Active,
            BOND_ARP_VALIDATE_BACKUP => Self::Backup,
            BOND_ARP_VALIDATE_ALL => Self::All,
            BOND_ARP_FILTER => Self::Filter,
            BOND_ARP_FILTER_ACTIVE => Self::FilterActive,
            BOND_ARP_FILTER_BACKUP => Self::FilterBackup,
            _ => Self::Other(d),
        }
    }
}

const BOND_PRI_RESELECT_ALWAYS: u8 = 0;
const BOND_PRI_RESELECT_BETTER: u8 = 1;
const BOND_PRI_RESELECT_FAILURE: u8 = 2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondPrimaryReselect {
    Always,
    Better,
    Failure,
    Other(u8),
}

impl From<u8> for BondPrimaryReselect {
    fn from(d: u8) -> Self {
        match d {
            BOND_PRI_RESELECT_ALWAYS => Self::Always,
            BOND_PRI_RESELECT_BETTER => Self::Better,
            BOND_PRI_RESELECT_FAILURE => Self::Failure,
            _ => Self::Other(d),
        }
    }
}

const BOND_FOM_NONE: u8 = 0;
const BOND_FOM_ACTIVE: u8 = 1;
const BOND_FOM_FOLLOW: u8 = 2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondFailOverMac {
    None,
    Active,
    Follow,
    Other(u8),
}

impl From<u8> for BondFailOverMac {
    fn from(d: u8) -> Self {
        match d {
            BOND_FOM_NONE => Self::None,
            BOND_FOM_ACTIVE => Self::Active,
            BOND_FOM_FOLLOW => Self::Follow,
            _ => Self::Other(d),
        }
    }
}

const BOND_XMIT_POLICY_LAYER2: u8 = 0;
const BOND_XMIT_POLICY_LAYER34: u8 = 1;
const BOND_XMIT_POLICY_LAYER23: u8 = 2;
const BOND_XMIT_POLICY_ENCAP23: u8 = 3;
const BOND_XMIT_POLICY_ENCAP34: u8 = 4;
const BOND_XMIT_POLICY_VLAN_SRCMAC: u8 = 5;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondXmitHashPolicy {
    #[serde(rename = "layer2")]
    Layer2,
    #[serde(rename = "layer3+4")]
    Layer34,
    #[serde(rename = "layer2+3")]
    Layer23,
    #[serde(rename = "encap2+3")]
    Encap23,
    #[serde(rename = "encap3+4")]
    Encap34,
    #[serde(rename = "vlan+srcmac")]
    VlanSrcMac,
    Other(u8),
}

impl From<u8> for BondXmitHashPolicy {
    fn from(d: u8) -> Self {
        match d {
            BOND_XMIT_POLICY_LAYER2 => Self::Layer2,
            BOND_XMIT_POLICY_LAYER34 => Self::Layer34,
            BOND_XMIT_POLICY_LAYER23 => Self::Layer23,
            BOND_XMIT_POLICY_ENCAP23 => Self::Encap23,
            BOND_XMIT_POLICY_ENCAP34 => Self::Encap34,
            BOND_XMIT_POLICY_VLAN_SRCMAC => Self::VlanSrcMac,
            _ => Self::Other(d),
        }
    }
}

const BOND_ALL_SUBORDINATES_ACTIVE_DROPPED: u8 = 0;
const BOND_ALL_SUBORDINATES_ACTIVE_DELIEVERD: u8 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondAllSubordinatesActive {
    Dropped,
    Delivered,
    Other(u8),
}

impl From<u8> for BondAllSubordinatesActive {
    fn from(d: u8) -> Self {
        match d {
            BOND_ALL_SUBORDINATES_ACTIVE_DROPPED => Self::Dropped,
            BOND_ALL_SUBORDINATES_ACTIVE_DELIEVERD => Self::Delivered,
            _ => Self::Other(d),
        }
    }
}

const AD_LACP_SLOW: u8 = 0;
const AD_LACP_FAST: u8 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondLacpRate {
    Slow,
    Fast,
    Other(u8),
}

impl From<u8> for BondLacpRate {
    fn from(d: u8) -> Self {
        match d {
            AD_LACP_SLOW => Self::Slow,
            AD_LACP_FAST => Self::Fast,
            _ => Self::Other(d),
        }
    }
}

const BOND_AD_STABLE: u8 = 0;
const BOND_AD_BANDWIDTH: u8 = 1;
const BOND_AD_COUNT: u8 = 2;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum BondAdSelect {
    Stable,
    Bandwidth,
    Count,
    Other(u8),
}

impl From<u8> for BondAdSelect {
    fn from(d: u8) -> Self {
        match d {
            BOND_AD_STABLE => Self::Stable,
            BOND_AD_BANDWIDTH => Self::Bandwidth,
            BOND_AD_COUNT => Self::Count,
            _ => Self::Other(d),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone, Default)]
#[non_exhaustive]
pub struct BondAdInfo {
    pub aggregator: u16,
    pub num_ports: u16,
    pub actor_key: u16,
    pub partner_key: u16,
    pub partner_mac: String,
}

impl From<&[nlas::BondAdInfo]> for BondAdInfo {
    fn from(nlas: &[nlas::BondAdInfo]) -> Self {
        let mut ret = Self::default();
        for nla in nlas {
            match nla {
                nlas::BondAdInfo::Aggregator(v) => ret.aggregator = *v,
                nlas::BondAdInfo::NumPorts(v) => ret.num_ports = *v,
                nlas::BondAdInfo::ActorKey(v) => ret.actor_key = *v,
                nlas::BondAdInfo::PartnerKey(v) => ret.partner_key = *v,
                nlas::BondAdInfo::PartnerMac(v) => {
                    match parse_as_mac(v.len(), v) {
                        Ok(m) => ret.partner_mac = m,
                        Err(e) => {
                            log::warn!(
                                "Failed to parse BondAdInfo.parse_as_mac: {}",
                                e
                            );
                        }
                    }
                }
                _ => {
                    log::warn!("Unknown BondAdInfo NLA {:?}", nla);
                }
            }
        }
        ret
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondInfo {
    pub subordinates: Vec<String>,
    pub mode: BondMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub miimon: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updelay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downdelay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_carrier: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_ip_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_all_targets: Option<BondModeArpAllTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_validate: Option<BondArpValidate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_reselect: Option<BondPrimaryReselect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fail_over_mac: Option<BondFailOverMac>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xmit_hash_policy: Option<BondXmitHashPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resend_igmp: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_unsol_na: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_grat_arp: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_subordinates_active: Option<BondAllSubordinatesActive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_links: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lp_interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packets_per_subordinate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lacp_rate: Option<BondLacpRate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_select: Option<BondAdSelect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_actor_sys_prio: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_user_port_key: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_actor_system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tlb_dynamic_lb: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_notif_delay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ad_info: Option<BondAdInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lacp_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_missed_max: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns_ip6_target: Option<Vec<Ipv6Addr>>,
}

impl From<&[InfoBond]> for BondInfo {
    fn from(nlas: &[InfoBond]) -> Self {
        let mut ret = Self::default();
        if let Some(mode) = nlas.iter().find_map(|nla| {
            if let InfoBond::Mode(v) = nla {
                Some(v)
            } else {
                None
            }
        }) {
            ret.mode = (*mode).into();
        }
        for nla in nlas {
            match nla {
                InfoBond::Mode(_) => (),
                InfoBond::MiiMon(v) => ret.miimon = Some(*v),
                InfoBond::UpDelay(v) => ret.updelay = Some(*v),
                InfoBond::DownDelay(v) => ret.downdelay = Some(*v),
                InfoBond::UseCarrier(v) => ret.use_carrier = Some(*v > 0),
                InfoBond::ArpInterval(v) => ret.arp_interval = Some(*v),
                InfoBond::ArpIpTarget(v) => {
                    ret.arp_ip_target = ipv4_addr_array_to_string(v).ok()
                }
                InfoBond::ArpValidate(v) => {
                    ret.arp_validate = Some((*v).into())
                }
                InfoBond::ArpAllTargets(v) => {
                    ret.arp_all_targets = Some((*v).into())
                }
                // For all the bond option limitation , please refer to
                // `bond_opts[BOND_OPT_LAST]` in linux kernel code:
                // `drivers/net/bonding/bond_options.c`
                InfoBond::Primary(v) => {
                    if [
                        BondMode::ActiveBackup,
                        BondMode::BalanceAlb,
                        BondMode::BalanceTlb,
                    ]
                    .contains(&ret.mode)
                    {
                        ret.primary = Some(format!("{v}"));
                    }
                }
                InfoBond::PrimaryReselect(v) => {
                    ret.primary_reselect = Some((*v).into())
                }
                InfoBond::FailOverMac(v) => {
                    if ret.mode == BondMode::ActiveBackup {
                        ret.fail_over_mac = Some((*v).into());
                    }
                }
                InfoBond::XmitHashPolicy(v) => {
                    if [
                        BondMode::BalanceXor,
                        BondMode::Ieee8021AD,
                        BondMode::BalanceTlb,
                    ]
                    .contains(&ret.mode)
                    {
                        ret.xmit_hash_policy = Some((*v).into());
                    }
                }
                InfoBond::ResendIgmp(v) => {
                    if [
                        BondMode::BalanceRoundRobin,
                        BondMode::ActiveBackup,
                        BondMode::BalanceTlb,
                        BondMode::BalanceAlb,
                    ]
                    .contains(&ret.mode)
                    {
                        ret.resend_igmp = Some(*v);
                    }
                }
                InfoBond::NumPeerNotif(v) => {
                    if ret.mode == BondMode::ActiveBackup {
                        ret.num_unsol_na = Some(*v);
                        ret.num_grat_arp = Some(*v);
                    }
                }
                InfoBond::AllPortsActive(v) => {
                    ret.all_subordinates_active = Some((*v).into())
                }
                // Kernel code has no limit on this, but document require
                // 802.3ad mode. Let's follow the kernel code here.
                InfoBond::MinLinks(v) => ret.min_links = Some(*v),
                // Kernel code has no limit on this, but document require
                // balance-tlb and balance-alb modes. Let's follow the kernel
                // code here.
                InfoBond::LpInterval(v) => ret.lp_interval = Some(*v),
                InfoBond::PacketsPerPort(v) => {
                    if ret.mode == BondMode::BalanceRoundRobin {
                        ret.packets_per_subordinate = Some(*v);
                    }
                }
                InfoBond::AdLacpRate(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.lacp_rate = Some((*v).into())
                    }
                }
                InfoBond::AdSelect(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.ad_select = Some((*v).into());
                    }
                }
                InfoBond::AdInfo(v) => ret.ad_info = Some(v.as_slice().into()),
                InfoBond::AdActorSysPrio(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.ad_actor_sys_prio = Some(*v);
                    }
                }
                InfoBond::AdUserPortKey(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.ad_user_port_key = Some(*v);
                    }
                }
                InfoBond::AdActorSystem(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.ad_actor_system = parse_as_mac(v.len(), v).ok();
                    }
                }
                InfoBond::TlbDynamicLb(v) => {
                    if [BondMode::BalanceTlb, BondMode::BalanceAlb]
                        .contains(&ret.mode)
                    {
                        ret.tlb_dynamic_lb = Some(*v > 0);
                    }
                }
                InfoBond::PeerNotifDelay(v) => ret.peer_notif_delay = Some(*v),
                InfoBond::AdLacpActive(v) => {
                    if ret.mode == BondMode::Ieee8021AD {
                        ret.lacp_active = Some(*v > 0);
                    }
                }
                InfoBond::MissedMax(v) => {
                    if ![
                        BondMode::Ieee8021AD,
                        BondMode::BalanceTlb,
                        BondMode::BalanceAlb,
                    ]
                    .contains(&ret.mode)
                    {
                        ret.arp_missed_max = Some(*v);
                    }
                }
                InfoBond::NsIp6Target(v) => ret.ns_ip6_target = Some(v.clone()),
                _ => {
                    log::warn!("Unsupported InfoBond: {:?}", nla);
                }
            }
        }
        ret
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BondSubordinateState {
    Active,
    Backup,
    Other(u8),
    Unknown,
}

const BOND_STATE_ACTIVE: u8 = 0;
const BOND_STATE_BACKUP: u8 = 1;

impl From<u8> for BondSubordinateState {
    fn from(d: u8) -> Self {
        match d {
            BOND_STATE_ACTIVE => Self::Active,
            BOND_STATE_BACKUP => Self::Backup,
            _ => Self::Other(d),
        }
    }
}

impl Default for BondSubordinateState {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum BondMiiStatus {
    LinkUp,
    LinkFail,
    LinkDown,
    LinkBack,
    Other(u8),
    Unknown,
}

const BOND_LINK_UP: u8 = 0;
const BOND_LINK_FAIL: u8 = 1;
const BOND_LINK_DOWN: u8 = 2;
const BOND_LINK_BACK: u8 = 3;

impl From<u8> for BondMiiStatus {
    fn from(d: u8) -> Self {
        match d {
            BOND_LINK_UP => Self::LinkUp,
            BOND_LINK_FAIL => Self::LinkFail,
            BOND_LINK_DOWN => Self::LinkDown,
            BOND_LINK_BACK => Self::LinkBack,
            _ => Self::Other(d),
        }
    }
}

impl Default for BondMiiStatus {
    fn default() -> Self {
        BondMiiStatus::Unknown
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondSubordinateInfo {
    pub subordinate_state: BondSubordinateState,
    pub mii_status: BondMiiStatus,
    pub link_failure_count: u32,
    pub perm_hwaddr: String,
    pub prio: i32,
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

pub(crate) fn get_bond_info(
    data: &InfoData,
) -> Result<Option<BondInfo>, NisporError> {
    if let InfoData::Bond(nlas) = data {
        Ok(Some(nlas.as_slice().into()))
    } else {
        Ok(None)
    }
}

pub(crate) fn get_bond_subordinate_info(
    nlas: &[InfoBondPort],
) -> Result<BondSubordinateInfo, NisporError> {
    let mut ret = BondSubordinateInfo::default();
    for nla in nlas {
        match nla {
            InfoBondPort::LinkFailureCount(d) => ret.link_failure_count = *d,
            InfoBondPort::MiiStatus(d) => ret.mii_status = u8::from(*d).into(),
            InfoBondPort::PermHwaddr(d) => {
                ret.perm_hwaddr = parse_as_mac(d.len(), d)?;
            }
            InfoBondPort::Prio(d) => ret.prio = *d,
            InfoBondPort::QueueId(d) => ret.queue_id = *d,
            InfoBondPort::BondPortState(d) => {
                ret.subordinate_state = u8::from(*d).into()
            }
            _ => {
                log::info!("Unknown bond port info {:?}", nla);
            }
        }
    }

    Ok(ret)
}

pub(crate) fn bond_iface_tidy_up(iface_states: &mut HashMap<String, Iface>) {
    gen_subordinate_list_of_controller(iface_states);
    primary_index_to_iface_name(iface_states);
}

fn gen_subordinate_list_of_controller(
    iface_states: &mut HashMap<String, Iface>,
) {
    let mut controller_subordinates: HashMap<String, Vec<String>> =
        HashMap::new();
    for iface in iface_states.values() {
        if iface.controller_type == Some(ControllerType::Bond) {
            if let Some(controller) = &iface.controller {
                match controller_subordinates.get_mut(controller) {
                    Some(subordinates) => subordinates.push(iface.name.clone()),
                    None => {
                        let new_subordinates: Vec<String> =
                            vec![iface.name.clone()];
                        controller_subordinates
                            .insert(controller.clone(), new_subordinates);
                    }
                };
            }
        }
    }
    for (controller, subordinates) in controller_subordinates.iter_mut() {
        if let Some(ref mut controller_iface) = iface_states.get_mut(controller)
        {
            if let Some(ref mut bond_info) = controller_iface.bond {
                subordinates.sort();
                bond_info.subordinates = subordinates.clone();
            }
        }
    }
}

fn primary_index_to_iface_name(iface_states: &mut HashMap<String, Iface>) {
    let mut index_to_name = HashMap::new();
    for iface in iface_states.values() {
        index_to_name.insert(format!("{}", iface.index), iface.name.clone());
    }
    for iface in iface_states.values_mut() {
        if iface.iface_type != IfaceType::Bond {
            continue;
        }
        if let Some(ref mut bond_info) = iface.bond {
            if let Some(index) = &bond_info.primary {
                if let Some(iface_name) = index_to_name.get(index) {
                    bond_info.primary = Some(iface_name.clone());
                }
            }
        }
    }
}

fn ipv4_addr_array_to_string(
    addrs: &[Ipv4Addr],
) -> Result<String, NisporError> {
    let mut rt = String::new();
    for i in 0..(addrs.len()) {
        let addr = &addrs.get(i).ok_or_else(|| {
            NisporError::bug("wrong index at parsing ipv4 as string".into())
        })?;
        rt.push_str(&addr.to_string());
        if i != addrs.len() - 1 {
            // This is how kernel sysfs is showing
            rt.push(',');
        }
    }
    Ok(rt)
}
