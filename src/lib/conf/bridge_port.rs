// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{packet_route::link::LinkMessage, LinkBridgePort};
use serde::{Deserialize, Serialize};

use crate::{
    BridgeMulticastRouterType, BridgePortStpState, BridgeVlanEntry, Iface,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct BridgePortConf {
    /// Flush the FDB if set to `true`.
    #[serde(default)]
    pub flush: bool,
    pub stp_state: Option<BridgePortStpState>,
    pub stp_priority: Option<u16>,
    pub stp_path_cost: Option<u32>,
    pub hairpin_mode: Option<bool>,
    pub bpdu_guard: Option<bool>,
    pub root_block: Option<bool>,
    pub multicast_fast_leave: Option<bool>,
    pub learning: Option<bool>,
    pub unicast_flood: Option<bool>,
    pub proxy_arp: Option<bool>,
    pub proxy_arp_wifi: Option<bool>,
    pub multicast_router: Option<BridgeMulticastRouterType>,
    pub multicast_flood: Option<bool>,
    pub multicast_to_unicast: Option<bool>,
    pub vlan_tunnel: Option<bool>,
    pub broadcast_flood: Option<bool>,
    pub group_fwd_mask: Option<u16>,
    pub neigh_suppress: Option<bool>,
    pub isolated: Option<bool>,
    pub mac_authentication_bypass: Option<bool>,
    pub backup_port: Option<u32>,
    pub locked: Option<bool>,
    pub neigh_vlan_suppress: Option<bool>,
    pub backup_nexthop_id: Option<u32>,
    pub vlans: Option<Vec<BridgeVlanEntry>>,
}

impl BridgePortConf {
    pub(crate) fn gen_port_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> LinkMessage {
        let mut builder = LinkBridgePort::new(cur_iface.index);

        if self.flush {
            builder = builder.fdb_flush();
        }

        if let Some(v) = self.stp_state {
            builder = builder.state(v.into());
        }

        if let Some(v) = self.stp_priority {
            builder = builder.priority(v);
        }

        if let Some(v) = self.stp_path_cost {
            builder = builder.cost(v);
        }

        if let Some(v) = self.hairpin_mode {
            builder = builder.hairpin(v);
        }

        if let Some(v) = self.bpdu_guard {
            builder = builder.guard(v);
        }

        if let Some(v) = self.root_block {
            builder = builder.root_block(v);
        }

        if let Some(v) = self.multicast_fast_leave {
            builder = builder.mcast_fast_leave(v);
        }

        if let Some(v) = self.learning {
            builder = builder.learning(v);
        }

        if let Some(v) = self.unicast_flood {
            builder = builder.flood(v);
        }

        if let Some(v) = self.proxy_arp {
            builder = builder.proxy_arp(v);
        }

        if let Some(v) = self.proxy_arp_wifi {
            builder = builder.proxy_arp_wifi(v);
        }

        if let Some(v) = self.multicast_router {
            builder = builder.mcast_router(v.into());
        }

        if let Some(v) = self.multicast_flood {
            builder = builder.mcast_flood(v);
        }

        if let Some(v) = self.multicast_to_unicast {
            builder = builder.mcast_to_unicast(v);
        }

        if let Some(v) = self.vlan_tunnel {
            builder = builder.vlan_tunnel(v);
        }

        if let Some(v) = self.broadcast_flood {
            builder = builder.bcast_flood(v);
        }

        if let Some(v) = self.group_fwd_mask {
            builder = builder.group_fwd_mask(v);
        }

        if let Some(v) = self.neigh_suppress {
            builder = builder.neigh_suppress(v);
        }

        if let Some(v) = self.isolated {
            builder = builder.isolated(v);
        }

        if let Some(v) = self.mac_authentication_bypass {
            builder = builder.mab(v);
        }

        if let Some(v) = self.backup_port {
            builder = builder.backup_port(v);
        }

        if let Some(v) = self.locked {
            builder = builder.locked(v);
        }

        if let Some(v) = self.neigh_vlan_suppress {
            builder = builder.neigh_vlan_suppress(v);
        }

        if let Some(v) = self.backup_nexthop_id {
            builder = builder.backup_nhid(v);
        }

        builder.build()
    }
}
