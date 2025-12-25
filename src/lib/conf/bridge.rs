// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkBridge, LinkMessageBuilder};
use serde::{Deserialize, Serialize};

use crate::{
    BridgeMulticastRouterType, BridgeStpState, IfaceConf, VlanProtocol,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct BridgeConf {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ageing_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_fwd_mask: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_address: Option<[u8; 6]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward_delay: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hello_time: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stp_state: Option<BridgeStpState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mst_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub no_linklocal_learn: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fdb_local_vlan_0: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fdb_max_learned: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_filtering: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_protocol: Option<VlanProtocol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_default_pvid: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vlan_stats_per_port: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_snooping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_vlan_snooping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_router: Option<BridgeMulticastRouterType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_query_use_ifaddr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_querier: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_hash_max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_last_member_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_startup_query_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_last_member_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_membership_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_querier_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_query_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_query_response_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_startup_query_interval: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_stats_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_igmp_version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcast_mld_version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_iptables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_ip6tables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nf_call_arptables: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mdb_offload_fail_notification: Option<bool>,
}

impl BridgeConf {
    pub(crate) fn gen_link_msg_builder(
        iface: &IfaceConf,
    ) -> LinkMessageBuilder<LinkBridge> {
        let mut builder = LinkBridge::new(iface.name.as_str());

        if let Some(br_conf) = iface.bridge.as_ref() {
            if let Some(v) = br_conf.ageing_time {
                builder = builder.ageing_time(v);
            }
            if let Some(v) = br_conf.group_fwd_mask {
                builder = builder.group_fwd_mask(v);
            }
            if let Some(v) = br_conf.group_address {
                builder = builder.group_address(v);
            }
            if let Some(v) = br_conf.forward_delay {
                builder = builder.forward_delay(v);
            }
            if let Some(v) = br_conf.hello_time {
                builder = builder.hello_time(v);
            }
            if let Some(v) = br_conf.max_age {
                builder = builder.max_age(v);
            }
            if let Some(v) = br_conf.stp_state.as_ref() {
                builder = builder.stp_state(v.clone().into());
            }
            if let Some(v) = br_conf.mst_enabled {
                builder = builder.mst_enabled(v);
            }
            if let Some(v) = br_conf.priority {
                builder = builder.priority(v);
            }
            if let Some(v) = br_conf.no_linklocal_learn {
                builder = builder.no_linklocal_learn(v);
            }
            if let Some(v) = br_conf.fdb_local_vlan_0 {
                builder = builder.fdb_local_vlan_0(v);
            }
            if let Some(v) = br_conf.fdb_max_learned {
                builder = builder.fdb_max_learned(v);
            }
            if let Some(v) = br_conf.vlan_filtering {
                builder = builder.vlan_filtering(v);
            }
            if let Some(v) = br_conf.vlan_protocol {
                builder = builder.vlan_protocol(v.into());
            }
            if let Some(v) = br_conf.vlan_default_pvid {
                builder = builder.vlan_default_pvid(v);
            }
            if let Some(v) = br_conf.vlan_stats_enabled {
                builder = builder.vlan_stats_enabled(v);
            }
            if let Some(v) = br_conf.vlan_stats_per_port {
                builder = builder.vlan_stats_per_port(v);
            }
            if let Some(v) = br_conf.mcast_snooping {
                builder = builder.mcast_snooping(v);
            }
            if let Some(v) = br_conf.mcast_vlan_snooping {
                builder = builder.mcast_vlan_snooping(v);
            }
            if let Some(v) = br_conf.mcast_router.as_ref() {
                builder = builder.mcast_router(v.clone().into());
            }
            if let Some(v) = br_conf.mcast_query_use_ifaddr {
                builder = builder.mcast_query_use_ifaddr(v);
            }
            if let Some(v) = br_conf.mcast_querier {
                builder = builder.mcast_querier(v);
            }
            if let Some(v) = br_conf.mcast_hash_max {
                builder = builder.mcast_hash_max(v);
            }
            if let Some(v) = br_conf.mcast_last_member_count {
                builder = builder.mcast_last_member_count(v);
            }
            if let Some(v) = br_conf.mcast_startup_query_count {
                builder = builder.mcast_startup_query_count(v);
            }
            if let Some(v) = br_conf.mcast_last_member_interval {
                builder = builder.mcast_last_member_interval(v);
            }
            if let Some(v) = br_conf.mcast_membership_interval {
                builder = builder.mcast_membership_interval(v);
            }
            if let Some(v) = br_conf.mcast_querier_interval {
                builder = builder.mcast_querier_interval(v);
            }
            if let Some(v) = br_conf.mcast_query_interval {
                builder = builder.mcast_query_interval(v);
            }
            if let Some(v) = br_conf.mcast_query_response_interval {
                builder = builder.mcast_query_response_interval(v);
            }
            if let Some(v) = br_conf.mcast_startup_query_interval {
                builder = builder.mcast_startup_query_interval(v);
            }
            if let Some(v) = br_conf.mcast_stats_enabled {
                builder = builder.mcast_stats_enabled(v);
            }
            if let Some(v) = br_conf.mcast_igmp_version {
                builder = builder.mcast_igmp_version(v);
            }
            if let Some(v) = br_conf.mcast_mld_version {
                builder = builder.mcast_mld_version(v);
            }
            if let Some(v) = br_conf.nf_call_iptables {
                builder = builder.nf_call_iptables(v);
            }
            if let Some(v) = br_conf.nf_call_ip6tables {
                builder = builder.nf_call_ip6tables(v);
            }
            if let Some(v) = br_conf.nf_call_arptables {
                builder = builder.nf_call_arptables(v);
            }
            if let Some(v) = br_conf.mdb_offload_fail_notification {
                builder = builder.mdb_offload_fail_notification(v);
            }
        }

        builder
    }
}
