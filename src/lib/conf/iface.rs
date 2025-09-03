// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use super::{
    super::query::get_ifaces_with_handle, alt_name::change_iface_alt_name,
    base_iface::apply_base_link_changes, ip::change_ip_layer,
};
use crate::{
    AltNameConf, BondConf, BridgeConf, ErrorKind, Iface, IfaceState, IfaceType,
    IpConf, NetStateIfaceFilter, NisporError, VethConf, VlanConf,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct IfaceConf {
    pub name: String,
    #[serde(default = "default_iface_state_in_conf")]
    pub state: IfaceState,
    #[serde(rename = "type")]
    pub iface_type: Option<IfaceType>,
    #[serde(default)]
    pub alt_names: Vec<AltNameConf>,
    /// Setting to Some(String::new()) will detach from controller
    pub controller: Option<String>,
    pub ipv4: Option<IpConf>,
    pub ipv6: Option<IpConf>,
    pub mac_address: Option<String>,
    pub veth: Option<VethConf>,
    pub bridge: Option<BridgeConf>,
    pub vlan: Option<VlanConf>,
    #[serde(alias = "link-aggregation")]
    pub bond: Option<BondConf>,
}

fn default_iface_state_in_conf() -> IfaceState {
    IfaceState::Up
}

pub(crate) async fn apply_iface_conf(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: Option<&Iface>,
) -> Result<(), NisporError> {
    if des_iface.state == IfaceState::Absent {
        if let Some(cur_iface) = cur_iface {
            delete_iface(handle, &des_iface.name, cur_iface.index).await?;
        } else {
            log::debug!(
                "Absent interface {} no found, no action",
                des_iface.name
            );
        }
    } else {
        if let Some(cur_iface) = cur_iface {
            change_iface(handle, des_iface, cur_iface).await?;
        } else {
            create_iface(handle, des_iface).await?;
            let mut iface_filter = NetStateIfaceFilter::minimum();
            iface_filter.iface_name = Some(des_iface.name.to_string());
            iface_filter.include_ip_address = true;
            let mut cur_ifaces =
                get_ifaces_with_handle(handle, Some(&iface_filter)).await?;
            let cur_iface =
                cur_ifaces.remove(&des_iface.name).ok_or_else(|| {
                    NisporError::new(
                        ErrorKind::NisporBug,
                        format!(
                            "Failed to find newly created interface \
                             {des_iface:?}"
                        ),
                    )
                })?;
            change_iface(handle, des_iface, &cur_iface).await?;
        }

        change_ip_layer(handle, des_iface).await?;
    }

    Ok(())
}

async fn create_iface(
    handle: &rtnetlink::Handle,
    iface: &IfaceConf,
) -> Result<(), NisporError> {
    let msg = match iface.iface_type {
        Some(IfaceType::Bridge) => BridgeConf::create(iface).build(),
        Some(IfaceType::Veth) => VethConf::create(iface)?.build(),
        Some(IfaceType::Bond) => BondConf::create(iface)?.build(),
        Some(IfaceType::Vlan) => VlanConf::create(handle, iface).await?.build(),
        Some(_) => {
            return Err(NisporError::invalid_argument(format!(
                "Cannot create unsupported interface {:?}",
                &iface
            )));
        }
        None => {
            return Err(NisporError::invalid_argument(format!(
                "No interface type defined for new interface {:?}",
                &iface
            )));
        }
    };
    handle.link().add(msg).execute().await.map_err(|e| {
        NisporError::new(
            ErrorKind::NisporBug,
            format!("Failed to create interface {iface:?}: {e}"),
        )
    })
}

async fn change_iface(
    handle: &rtnetlink::Handle,
    des_iface: &IfaceConf,
    cur_iface: &Iface,
) -> Result<(), NisporError> {
    // TODO: Change link layer settings(bond, veth, bridg, etc)
    apply_base_link_changes(handle, des_iface, cur_iface).await?;

    change_iface_alt_name(handle, des_iface, cur_iface).await?;
    Ok(())
}

async fn delete_iface(
    handle: &rtnetlink::Handle,
    iface_name: &str,
    index: u32,
) -> Result<(), NisporError> {
    handle.link().del(index).execute().await.map_err(|e| {
        NisporError::new(
            ErrorKind::NisporBug,
            format!("Failed to delete interface {iface_name}: {e}"),
        )
    })
}
