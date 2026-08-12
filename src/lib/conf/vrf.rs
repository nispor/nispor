// SPDX-License-Identifier: Apache-2.0

use rtnetlink::{LinkMessageBuilder, LinkVrf, packet_route::link::InfoKind};
use serde::{Deserialize, Serialize};

use crate::{ErrorKind, Iface, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
#[non_exhaustive]
pub struct VrfConf {
    pub table_id: Option<u32>,
}

impl VrfConf {
    pub(crate) fn gen_link_msg_builder(
        iface: &IfaceConf,
        cur_iface: Option<&Iface>,
    ) -> Result<LinkMessageBuilder<LinkVrf>, NisporError> {
        let mut builder =
            LinkMessageBuilder::<LinkVrf>::new_with_info_kind(InfoKind::Vrf)
                .name(iface.name.to_string());

        let Some(table_id) = iface.vrf.as_ref().and_then(|c| c.table_id) else {
            if cur_iface.is_none() {
                return Err(NisporError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "Need table ID for creating new VRF {}",
                        iface.name
                    ),
                ));
            }
            return Ok(builder);
        };

        if table_id == 0 {
            return Err(NisporError::new(
                ErrorKind::InvalidArgument,
                format!("Invalid VRF table ID 0 for {}", iface.name),
            ));
        }

        if let Some(cur_iface) = cur_iface {
            if cur_iface.vrf.as_ref().map(|v| v.table_id) != Some(table_id) {
                return Err(NisporError::new(
                    ErrorKind::InvalidArgument,
                    format!(
                        "VRF table ID of {} cannot be changed, please delete \
                         and recreate the VRF interface",
                        iface.name
                    ),
                ));
            }
            // The kernel rejects IFLA_VRF_TABLE on an existing VRF
            // (changelink is not supported), so skip it when the table ID
            // is unchanged to keep re-applying the same config a no-op.
        } else {
            builder = builder.table_id(table_id);
        }

        Ok(builder)
    }
}

#[cfg(test)]
mod tests {
    use rtnetlink::packet_route::link::{
        InfoData, InfoVrf, LinkAttribute, LinkInfo, LinkMessage,
    };

    use super::*;
    use crate::{Iface, IfaceConf, IfaceType, VrfInfo};

    fn vrf_iface_conf(table_id: Option<u32>) -> IfaceConf {
        IfaceConf {
            name: "vrf0".to_string(),
            iface_type: Some(IfaceType::Vrf),
            vrf: Some(VrfConf { table_id }),
            ..Default::default()
        }
    }

    fn vrf_iface(table_id: u32) -> Iface {
        Iface {
            name: "vrf0".to_string(),
            vrf: Some(VrfInfo {
                table_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn has_vrf_info_data(msg: &LinkMessage) -> bool {
        msg.attributes.iter().any(|attr| match attr {
            LinkAttribute::LinkInfo(infos) => infos.iter().any(|info| {
                matches!(
                    info,
                    LinkInfo::Data(InfoData::Vrf(vrf_infos))
                        if vrf_infos.iter().any(|v| {
                            matches!(v, InfoVrf::TableId(_))
                        })
                )
            }),
            _ => false,
        })
    }

    #[test]
    fn new_vrf_includes_table_id() {
        let msg =
            VrfConf::gen_link_msg_builder(&vrf_iface_conf(Some(10)), None)
                .unwrap()
                .build();

        assert!(has_vrf_info_data(&msg));
    }

    #[test]
    fn existing_vrf_with_same_table_id_skips_info_data() {
        let msg = VrfConf::gen_link_msg_builder(
            &vrf_iface_conf(Some(10)),
            Some(&vrf_iface(10)),
        )
        .unwrap()
        .build();

        assert!(!has_vrf_info_data(&msg));
    }

    #[test]
    fn existing_vrf_with_different_table_id_rejected() {
        let err = VrfConf::gen_link_msg_builder(
            &vrf_iface_conf(Some(20)),
            Some(&vrf_iface(10)),
        )
        .unwrap_err();

        assert!(matches!(err.kind, ErrorKind::InvalidArgument));
        assert!(err.msg.contains("delete and recreate"));
    }

    #[test]
    fn new_vrf_without_table_id_rejected() {
        let confs = [
            IfaceConf {
                name: "vrf0".to_string(),
                iface_type: Some(IfaceType::Vrf),
                vrf: None,
                ..Default::default()
            },
            vrf_iface_conf(None),
        ];

        for conf in confs {
            let err = VrfConf::gen_link_msg_builder(&conf, None).unwrap_err();
            assert!(matches!(err.kind, ErrorKind::InvalidArgument));
        }
    }

    #[test]
    fn zero_table_id_rejected() {
        let err = VrfConf::gen_link_msg_builder(&vrf_iface_conf(Some(0)), None)
            .unwrap_err();

        assert!(matches!(err.kind, ErrorKind::InvalidArgument));
    }
}
