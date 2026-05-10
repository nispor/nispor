// SPDX-License-Identifier: Apache-2.0

use rtnetlink::new_connection;

use super::iface::apply_iface_conf;
#[cfg(feature = "wireguard")]
use super::wireguard::apply_wg_conf;
#[cfg(feature = "wireguard")]
use crate::IfaceType;
use crate::{IfaceConf, NisporError};

pub(crate) async fn apply_ifaces_conf(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for des_iface in des_ifaces {
        apply_iface_conf(&handle, des_iface).await?;
    }

    #[cfg(feature = "wireguard")]
    apply_wireguard_confs(des_ifaces).await?;

    Ok(())
}

#[cfg(feature = "wireguard")]
async fn apply_wireguard_confs(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let wg_ifaces: Vec<_> = des_ifaces
        .iter()
        .filter(|i| i.iface_type == Some(IfaceType::Wireguard))
        .collect();

    if wg_ifaces.is_empty() {
        return Ok(());
    }

    let (wg_connection, mut wg_handle, _) = nl_wireguard::new_connection()?;
    tokio::spawn(wg_connection);

    for des_iface in wg_ifaces {
        apply_wg_conf(&mut wg_handle, des_iface).await?;
    }

    Ok(())
}
