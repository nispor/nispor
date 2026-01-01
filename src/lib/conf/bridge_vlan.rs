// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_route::link::{BridgeVlanInfoFlags, LinkMessage},
    LinkBridgeVlan, LinkMessageBuilder,
};

use crate::{BridgeConf, BridgePortConf, BridgeVlanEntry, Iface};

fn add_bridge_vlan_to_builder<'a, T>(
    mut builder: LinkMessageBuilder<LinkBridgeVlan>,
    vlans: T,
) -> LinkMessageBuilder<LinkBridgeVlan>
where
    T: Iterator<Item = &'a BridgeVlanEntry>,
{
    for vlan in vlans {
        let mut flag = BridgeVlanInfoFlags::empty();
        if vlan.is_pvid {
            flag |= BridgeVlanInfoFlags::Pvid;
        }
        if vlan.is_egress_untagged {
            flag |= BridgeVlanInfoFlags::Untagged;
        }
        if let Some(vid) = vlan.vid {
            builder = builder.vlan(vid, flag);
        } else if let Some((vid_start, vid_end)) = vlan.vid_range.as_ref() {
            builder = builder
                .vlan_range_start(*vid_start, flag)
                .vlan_range_end(*vid_end, flag);
        }
    }
    builder
}

impl BridgePortConf {
    pub(crate) fn gen_add_port_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        let vlans = self.vlans.as_ref()?;
        let mut vlans_to_add = vlans.iter().filter(|v| !v.remove).peekable();
        vlans_to_add.peek()?;
        let builder = add_bridge_vlan_to_builder(
            LinkBridgeVlan::new(cur_iface.index),
            vlans_to_add,
        );
        Some(builder.build())
    }

    pub(crate) fn gen_del_port_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        let vlans = self.vlans.as_ref()?;
        let mut vlans_to_del = vlans.iter().filter(|v| v.remove).peekable();
        vlans_to_del.peek()?;
        let builder = add_bridge_vlan_to_builder(
            LinkBridgeVlan::new(cur_iface.index),
            vlans_to_del,
        );
        Some(builder.build())
    }
}

impl BridgeConf {
    pub(crate) fn gen_add_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        let vlans = self.vlans.as_ref()?;
        let mut vlans_to_add = vlans.iter().filter(|v| !v.remove).peekable();
        vlans_to_add.peek()?;
        let builder = add_bridge_vlan_to_builder(
            LinkBridgeVlan::new(cur_iface.index).bridge_self(),
            vlans_to_add,
        );
        Some(builder.build())
    }

    pub(crate) fn gen_del_vlan_conf_link_msg(
        &self,
        cur_iface: &Iface,
    ) -> Option<LinkMessage> {
        let vlans = self.vlans.as_ref()?;
        let mut vlans_to_del = vlans.iter().filter(|v| v.remove).peekable();
        vlans_to_del.peek()?;
        let builder = add_bridge_vlan_to_builder(
            LinkBridgeVlan::new(cur_iface.index).bridge_self(),
            vlans_to_del,
        );
        Some(builder.build())
    }
}
