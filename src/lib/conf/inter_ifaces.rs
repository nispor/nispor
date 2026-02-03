// SPDX-License-Identifier: Apache-2.0

use rtnetlink::new_connection;

use super::{iface::apply_iface_conf, wireguard::apply_wg_conf};
use crate::{IfaceConf, IfaceType, NisporError};

pub(crate) async fn apply_ifaces_conf(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for des_iface in des_ifaces {
        apply_iface_conf(&handle, des_iface).await?;
    }

    let wg_ifaces: Vec<_> = des_ifaces
        .iter()
        .filter(|i| i.iface_type == Some(IfaceType::Wireguard))
        .collect();

    if !wg_ifaces.is_empty() {
        let (wg_connection, mut wg_handle, _) = nl_wireguard::new_connection()?;
        tokio::spawn(wg_connection);

        for des_iface in wg_ifaces {
            apply_wg_conf(&mut wg_handle, des_iface).await?;
        }
    }

    Ok(())
}
