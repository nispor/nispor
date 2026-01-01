// SPDX-License-Identifier: Apache-2.0

use std::net::{Ipv4Addr, Ipv6Addr};

use rtnetlink::{LinkBond, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    mac::{mac_str_to_raw, ETH_ALEN},
    query::resolve_iface_index,
    BondAdSelect, BondAllPortsActive, BondArpValidate, BondFailOverMac,
    BondLacpRate, BondMode, BondModeArpAllTargets, BondPrimaryReselect,
    BondXmitHashPolicy, ErrorKind, Iface, IfaceConf, NisporError,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BondConf {
    pub mode: Option<BondMode>,
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
    pub arp_ip_target: Option<Vec<Ipv4Addr>>,
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
    #[serde(skip_serializing_if = "Option::is_none", alias = "num_grat_arp")]
    pub num_unsol_na: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub all_ports_active: Option<BondAllPortsActive>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_links: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lp_interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packets_per_port: Option<u32>,
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
    pub lacp_active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arp_missed_max: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ns_ip6_target: Option<Vec<Ipv6Addr>>,
}

impl BondConf {
    pub(crate) async fn gen_link_msg_builder(
        handle: &rtnetlink::Handle,
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<LinkMessageBuilder<LinkBond>, NisporError> {
        let mut builder = LinkBond::new(iface.name.as_str());
        if let Some(bond_conf) = iface.bond.as_ref() {
            if let Some(bond_mode) = bond_conf.mode {
                let cur_bond_mode =
                    cur_iface.and_then(|i| i.bond.as_ref()).map(|b| b.mode);

                // Only include bond mode request when changing, otherwise
                // kernel will reject us.
                if Some(bond_mode) != cur_bond_mode {
                    builder = builder.mode(bond_mode.into());
                }
            }
            if let Some(v) = bond_conf.miimon {
                builder = builder.miimon(v);
            }
            if let Some(v) = bond_conf.updelay {
                builder = builder.updelay(v);
            }
            if let Some(v) = bond_conf.downdelay {
                builder = builder.downdelay(v);
            }
            if let Some(v) = bond_conf.use_carrier {
                builder = builder.use_carrier(v);
            }
            if let Some(v) = bond_conf.arp_interval {
                builder = builder.arp_interval(v);
            }
            if let Some(v) = bond_conf.arp_ip_target.as_ref() {
                builder = builder.arp_ip_target(v.to_vec());
            }
            if let Some(v) = bond_conf.arp_all_targets {
                builder = builder.arp_all_targets(v.into());
            }
            if let Some(v) = bond_conf.arp_validate {
                builder = builder.arp_validate(v.into());
            }
            if let Some(v) = bond_conf.primary.as_deref() {
                builder =
                    builder.primary(resolve_iface_index(handle, v).await?);
            }
            if let Some(v) = bond_conf.primary_reselect {
                builder = builder.primary_reselect(v.into());
            }
            if let Some(v) = bond_conf.fail_over_mac {
                builder = builder.fail_over_mac(v.into());
            }
            if let Some(v) = bond_conf.xmit_hash_policy {
                builder = builder.xmit_hash_policy(v.into());
            }
            if let Some(v) = bond_conf.resend_igmp {
                builder = builder.resend_igmp(v);
            }
            if let Some(v) = bond_conf.num_unsol_na {
                builder = builder.num_peer_notif(v);
            }
            if let Some(v) = bond_conf.all_ports_active {
                builder = builder.all_ports_active(v.into());
            }
            if let Some(v) = bond_conf.min_links {
                builder = builder.min_links(v);
            }
            if let Some(v) = bond_conf.lp_interval {
                builder = builder.lp_interval(v);
            }
            if let Some(v) = bond_conf.packets_per_port {
                builder = builder.packets_per_port(v);
            }
            if let Some(v) = bond_conf.lacp_rate {
                builder = builder.ad_lacp_rate(v.into());
            }
            if let Some(v) = bond_conf.ad_select {
                builder = builder.ad_select(v.into());
            }
            if let Some(v) = bond_conf.ad_actor_sys_prio {
                builder = builder.ad_actor_sys_prio(v);
            }
            if let Some(v) = bond_conf.ad_user_port_key {
                builder = builder.ad_user_port_key(v);
            }
            if let Some(v) = bond_conf.ad_actor_system.as_deref() {
                let mac = mac_str_to_raw(v)?;
                if mac.len() == ETH_ALEN {
                    builder = builder.ad_actor_system(mac.try_into().map_err(
                        |e| {
                            NisporError::new(
                                ErrorKind::InvalidArgument,
                                format!(
                                    "Invalid bond ad_actor_system: {:?}",
                                    e
                                ),
                            )
                        },
                    )?);
                } else {
                    return Err(NisporError::new(
                        ErrorKind::InvalidArgument,
                        format!(
                            "Invalid mac address length for bond \
                             ad_actor_system {v}, should be like \
                             01:23:45:67:89:ab"
                        ),
                    ));
                }
            }
            if let Some(v) = bond_conf.tlb_dynamic_lb {
                builder = builder.tlb_dynamic_lb(v);
            }
            if let Some(v) = bond_conf.peer_notif_delay {
                builder = builder.peer_notif_delay(v);
            }
            if let Some(v) = bond_conf.lacp_active {
                builder = builder.ad_lacp_active(v);
            }
            if let Some(v) = bond_conf.arp_missed_max {
                builder = builder.missed_max(v);
            }
            if let Some(v) = bond_conf.ns_ip6_target.as_ref() {
                builder = builder.ns_ip6_target(v.to_vec());
            }
        }

        Ok(builder)
    }
}
