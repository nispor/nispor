// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{
    Handle, LinkMessageBuilder, LinkVlan,
    packet_route::link::{InfoKind, VlanFlags},
};
use serde::{Deserialize, Serialize};

use super::super::query::resolve_iface_index;
use crate::{
    ErrorKind, Iface, IfaceConf, NisporError, VlanProtocol, VlanQosMapping,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct VlanConf {
    pub vlan_id: Option<u16>,
    pub base_iface: Option<String>,
    pub protocol: Option<VlanProtocol>,
    pub is_reorder_hdr: Option<bool>,
    pub is_gvrp: Option<bool>,
    pub is_loose_binding: Option<bool>,
    pub is_mvrp: Option<bool>,
    pub is_bridge_binding: Option<bool>,
    #[serde(default)]
    pub ingress_qos_map: Option<Vec<VlanQosMapping>>,
    #[serde(default)]
    pub egress_qos_map: Option<Vec<VlanQosMapping>>,
}

impl VlanConf {
    pub(crate) async fn gen_link_msg_builder(
        handle: &Handle,
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<LinkMessageBuilder<LinkVlan>, NisporError> {
        let mut builder =
            LinkMessageBuilder::<LinkVlan>::new_with_info_kind(InfoKind::Vlan)
                .name(iface.name.to_string());
        if let Some(vlan_conf) = iface.vlan.as_ref() {
            if cur_iface.is_none()
                && (vlan_conf.vlan_id.is_none()
                    || vlan_conf.base_iface.is_none())
            {
                return Err(NisporError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Need VLAN id and base_iface for creating new VLAN {}",
                        iface.name
                    ),
                ));
            }
            if let Some(parent) = vlan_conf.base_iface.as_ref() {
                // We have to query the interface index here because it might
                // just been created before us.
                let parent_index = resolve_iface_index(handle, parent).await?;
                builder = builder.link(parent_index);
            }
            if let Some(id) = vlan_conf.vlan_id {
                builder = builder.id(id);
            }
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

            let cur_vlan_info = cur_iface.and_then(|i| i.vlan.as_ref());

            let effective_ingress: Vec<VlanQosMapping> =
                if let Some(desired) = vlan_conf.ingress_qos_map.as_ref() {
                    let mut result = desired.clone();
                    // Kernel only sets/overwrites individual entries on
                    // changelink; to remove an entry we must explicitly
                    // set its `to` to 0.
                    if let Some(cur_info) = cur_vlan_info {
                        for cur_map in &cur_info.ingress_qos_map {
                            if !desired.iter().any(|d| d.from == cur_map.from) {
                                result.push(VlanQosMapping {
                                    from: cur_map.from,
                                    to: 0,
                                });
                            }
                        }
                    }
                    result
                } else {
                    Vec::new()
                };

            let effective_egress: Vec<VlanQosMapping> =
                if let Some(desired) = vlan_conf.egress_qos_map.as_ref() {
                    let mut result = desired.clone();
                    if let Some(cur_info) = cur_vlan_info {
                        for cur_map in &cur_info.egress_qos_map {
                            if !desired.iter().any(|d| d.from == cur_map.from) {
                                result.push(VlanQosMapping {
                                    from: cur_map.from,
                                    to: 0,
                                });
                            }
                        }
                    }
                    result
                } else {
                    Vec::new()
                };

            if !effective_ingress.is_empty() || !effective_egress.is_empty() {
                builder = builder.qos(
                    effective_ingress.iter().map(|m| m.into()),
                    effective_egress.iter().map(|m| m.into()),
                );
            }
        }
        Ok(builder)
    }
}
