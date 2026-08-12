// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use rtnetlink::{
    Handle, LinkMessageBuilder, LinkVxlan, packet_route::link::InfoVxlan,
};
use serde::{Deserialize, Serialize};

use super::super::query::resolve_iface_index;
use crate::{ErrorKind, Iface, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct VxlanConf {
    /// VNI, cannot changed after creation
    pub vxlan_id: Option<u32>,
    pub base_iface: Option<String>,
    pub remote: Option<IpAddr>,
    pub local: Option<IpAddr>,
    /// cannot changed after creation
    pub dst_port: Option<u16>,
    pub learning: Option<bool>,
    pub ttl: Option<u8>,
    pub tos: Option<u8>,
    pub ageing: Option<u32>,
    /// cannot changed after creation
    pub max_address: Option<u32>,
    /// cannot changed after creation
    pub src_port_min: Option<u16>,
    /// cannot changed after creation
    pub src_port_max: Option<u16>,
    /// cannot changed after creation
    pub proxy: Option<bool>,
    /// cannot changed after creation
    pub rsc: Option<bool>,
    /// cannot changed after creation
    pub l2miss: Option<bool>,
    /// cannot changed after creation
    pub l3miss: Option<bool>,
    /// cannot changed after creation
    pub udp_check_sum: Option<bool>,
    /// cannot changed after creation
    pub udp6_zero_check_sum_tx: Option<bool>,
    /// cannot changed after creation
    pub udp6_zero_check_sum_rx: Option<bool>,
    /// cannot changed after creation
    pub remote_check_sum_tx: Option<bool>,
    /// cannot changed after creation
    pub remote_check_sum_rx: Option<bool>,
    /// cannot changed after creation
    pub gbp: Option<bool>,
    /// cannot changed after creation
    pub remote_check_sum_no_partial: Option<bool>,
    /// cannot changed after creation
    pub collect_metadata: Option<bool>,
    /// For IPv6 only
    pub label: Option<u32>,
    /// cannot changed after creation
    pub gpe: Option<bool>,
    /// cannot changed after creation
    pub ttl_inherit: Option<bool>,
    // TODO: df: set/unset/inherit
    // TODO: reserved_bits (cannot changed after creation)
    // TODO: localbypass
    // TODO: mcroute
}

impl VxlanConf {
    pub(crate) async fn gen_link_msg_builder(
        handle: &Handle,
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<LinkMessageBuilder<LinkVxlan>, NisporError> {
        let Some(vxlan_conf) = iface.vxlan.as_ref() else {
            return Err(NisporError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Missing vxlan section for creating new VxLAN {}",
                    iface.name
                ),
            ));
        };

        let mut builder = if let Some(vxlan_id) = vxlan_conf.vxlan_id {
            LinkVxlan::new(&iface.name, vxlan_id)
        } else {
            if cur_iface.is_none() {
                return Err(NisporError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Missing vxlan.id for creating new VxLAN {}",
                        iface.name
                    ),
                ));
            } else {
                LinkMessageBuilder::<LinkVxlan>::new(&iface.name)
            }
        };

        if let Some(parent) = vxlan_conf.base_iface.as_ref() {
            let parent_index = resolve_iface_index(handle, parent).await?;
            builder = builder.dev(parent_index);
        }

        if let Some(remote) = vxlan_conf.remote.as_ref() {
            match remote {
                IpAddr::V4(v4) => {
                    builder = builder.remote(*v4);
                }
                IpAddr::V6(v6) => {
                    builder = builder.remote6(*v6);
                }
            }
        }

        if let Some(local) = vxlan_conf.local.as_ref() {
            match local {
                IpAddr::V4(v4) => {
                    builder = builder.local(*v4);
                }
                IpAddr::V6(v6) => {
                    builder = builder.local6(*v6);
                }
            }
        }

        if let Some(d) = vxlan_conf.dst_port {
            builder = builder.port(d);
        }

        if let Some(d) = vxlan_conf.learning {
            builder = builder.learning(d);
        }

        if let Some(d) = vxlan_conf.ttl {
            builder = builder.ttl(d);
        }

        if let Some(d) = vxlan_conf.tos {
            builder = builder.tos(d);
        }

        if let Some(d) = vxlan_conf.ageing {
            builder = builder.ageing(d);
        }

        if let Some(d) = vxlan_conf.max_address {
            builder = builder.limit(d);
        }

        if let (Some(min), Some(max)) =
            (vxlan_conf.src_port_min, vxlan_conf.src_port_max)
        {
            builder = builder.port_range(min, max);
        }

        if let Some(d) = vxlan_conf.proxy {
            builder = builder.proxy(d);
        }

        if let Some(d) = vxlan_conf.rsc {
            builder = builder.rsc(d);
        }

        if let Some(d) = vxlan_conf.l2miss {
            builder = builder.l2miss(d);
        }

        if let Some(d) = vxlan_conf.l3miss {
            builder = builder.l3miss(d);
        }

        if let Some(d) = vxlan_conf.udp_check_sum {
            builder = builder.udp_csum(d);
        }

        if let Some(d) = vxlan_conf.udp6_zero_check_sum_tx {
            builder = builder.append_info_data(InfoVxlan::UDPZeroCsumTX(d));
        }

        if let Some(d) = vxlan_conf.udp6_zero_check_sum_rx {
            builder = builder.append_info_data(InfoVxlan::UDPZeroCsumRX(d));
        }

        if let Some(d) = vxlan_conf.remote_check_sum_tx {
            builder = builder.append_info_data(InfoVxlan::RemCsumTX(d));
        }

        if let Some(d) = vxlan_conf.remote_check_sum_rx {
            builder = builder.append_info_data(InfoVxlan::RemCsumRX(d));
        }

        if let Some(true) = vxlan_conf.gbp {
            builder = builder.append_info_data(InfoVxlan::Gbp);
        }

        if let Some(true) = vxlan_conf.remote_check_sum_no_partial {
            builder = builder.append_info_data(InfoVxlan::RemCsumNoPartial);
        }

        if let Some(d) = vxlan_conf.collect_metadata {
            builder = builder.collect_metadata(d);
        }

        if let Some(d) = vxlan_conf.label {
            builder = builder.label(d);
        }

        if let Some(true) = vxlan_conf.gpe {
            builder = builder.append_info_data(InfoVxlan::Gpe);
        }

        if let Some(true) = vxlan_conf.ttl_inherit {
            builder = builder.append_info_data(InfoVxlan::TtlInheritFlag);
        }

        Ok(builder)
    }
}
