// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    packet_route::link::VlanFlags, Handle, LinkMessageBuilder, LinkVlan,
};
use serde::{Deserialize, Serialize};

use super::super::query::resolve_iface_index;
use crate::{ErrorKind, IfaceConf, NisporError, VlanProtocol, VlanQosMapping};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct VlanConf {
    pub vlan_id: u16,
    pub base_iface: String,
    pub protocol: Option<VlanProtocol>,
    pub is_reorder_hdr: Option<bool>,
    pub is_gvrp: Option<bool>,
    pub is_loose_binding: Option<bool>,
    pub is_mvrp: Option<bool>,
    pub is_bridge_binding: Option<bool>,
    #[serde(default)]
    pub ingress_qos_map: Vec<VlanQosMapping>,
    #[serde(default)]
    pub egress_qos_map: Vec<VlanQosMapping>,
}

impl VlanConf {
    pub(crate) async fn create(
        handle: &Handle,
        iface: &IfaceConf,
    ) -> Result<LinkMessageBuilder<LinkVlan>, NisporError> {
        if let Some(vlan_conf) = iface.vlan.as_ref() {
            let parent_index =
                resolve_iface_index(handle, &vlan_conf.base_iface).await?;
            let mut builder = LinkVlan::new(
                iface.name.as_str(),
                parent_index,
                vlan_conf.vlan_id,
            );
            if let Some(protocol) = vlan_conf.protocol {
                builder = builder.protocol(protocol.into());
            }
            let mut flags = VlanFlags::empty();
            let mut flags_mask = VlanFlags::empty();

            let mut set_flag = |v, flag| {
                if let Some(val) = v {
                    flags_mask |= flag;
                    if val {
                        flags |= flag;
                    }
                }
            };
            set_flag(vlan_conf.is_reorder_hdr, VlanFlags::ReorderHdr);
            set_flag(vlan_conf.is_gvrp, VlanFlags::Gvrp);
            set_flag(vlan_conf.is_loose_binding, VlanFlags::LooseBinding);
            set_flag(vlan_conf.is_mvrp, VlanFlags::Mvrp);
            set_flag(vlan_conf.is_bridge_binding, VlanFlags::BridgeBinding);

            if flags_mask != VlanFlags::empty() {
                builder = builder.flags(flags, flags_mask);
            }

            builder = builder.qos(
                vlan_conf.ingress_qos_map.iter().map(|m| m.into()),
                vlan_conf.egress_qos_map.iter().map(|m| m.into()),
            );

            Ok(builder)
        } else {
            Err(NisporError::new(
                ErrorKind::NisporBug,
                format!("No vlan section defined for creating VLAN {iface:?}"),
            ))
        }
    }
}
