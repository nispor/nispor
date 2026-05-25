// SPDX-License-Identifier: Apache-2.0

use std::net::IpAddr;

use rtnetlink::{
    Handle, LinkVxlan,
    packet_route::link::{InfoVxlan, LinkMessage},
};
use serde::{Deserialize, Serialize};

use super::super::query::resolve_iface_index;
use crate::{ErrorKind, Iface, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct VxlanConf {
    pub vxlan_id: Option<u32>,
    pub base_iface: Option<String>,
    pub remote: Option<IpAddr>,
    pub local: Option<IpAddr>,
    pub dst_port: Option<u16>,
    pub learning: Option<bool>,
    pub ttl: Option<u8>,
    pub tos: Option<u8>,
    pub ageing: Option<u32>,
    pub max_address: Option<u32>,
    pub src_port_min: Option<u16>,
    pub src_port_max: Option<u16>,
    pub proxy: Option<bool>,
    pub rsc: Option<bool>,
    pub l2miss: Option<bool>,
    pub l3miss: Option<bool>,
    pub udp_check_sum: Option<bool>,
    pub udp6_zero_check_sum_tx: Option<bool>,
    pub udp6_zero_check_sum_rx: Option<bool>,
    pub remote_check_sum_tx: Option<bool>,
    pub remote_check_sum_rx: Option<bool>,
    pub gbp: Option<bool>,
    pub remote_check_sum_no_partial: Option<bool>,
    pub collect_metadata: Option<bool>,
    /// For IPv6 only
    pub label: Option<u32>,
    pub gpe: Option<bool>,
    pub ttl_inherit: Option<bool>,
    // TODO: df: set/unset/inherit
}

impl VxlanConf {
    pub(crate) async fn gen_link_msg_builder(
        handle: &Handle,
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<Vec<LinkMessage>, NisporError> {
        let Some(vxlan_conf) = iface.vxlan.as_ref() else {
            return Ok(Vec::new());
        };

        if cur_iface.is_some() {
            return Ok(Vec::new());
        }

        let vxlan_id = vxlan_conf.vxlan_id.ok_or_else(|| {
            NisporError::new(
                ErrorKind::InvalidArgument,
                format!(
                    "Missing vxlan.id for creating new VxLAN {}",
                    iface.name
                ),
            )
        })?;

        let mut builder = LinkVxlan::new(&iface.name, vxlan_id);

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
            builder = builder.append_info_data(InfoVxlan::Gbp(true));
        }

        if let Some(true) = vxlan_conf.remote_check_sum_no_partial {
            builder =
                builder.append_info_data(InfoVxlan::RemCsumNoPartial(true));
        }

        if let Some(d) = vxlan_conf.collect_metadata {
            builder = builder.collect_metadata(d);
        }

        if let Some(d) = vxlan_conf.label {
            builder = builder.label(d);
        }

        if let Some(true) = vxlan_conf.gpe {
            builder = builder.append_info_data(InfoVxlan::Gpe(true));
        }

        if let Some(d) = vxlan_conf.ttl_inherit {
            builder = builder.append_info_data(InfoVxlan::TtlInherit(d));
        }

        Ok(vec![builder.build()])
    }
}
