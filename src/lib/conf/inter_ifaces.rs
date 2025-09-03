// SPDX-License-Identifier: Apache-2.0

use rtnetlink::new_connection;

use super::{super::query::get_ifaces_with_handle, iface::apply_iface_conf};
use crate::{IfaceConf, NetStateIfaceFilter, NisporError};

pub(crate) async fn apply_ifaces_conf(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    // Instead of query full current state, we query desired interface only
    // one by one, because:
    // 1. Consistent performance regardless current interface amount.
    // 2. Allowing removing interface and add it back sequentially.
    for des_iface in des_ifaces {
        let mut iface_filter = NetStateIfaceFilter::minimum();
        iface_filter.iface_name = Some(des_iface.name.to_string());
        if des_iface.ipv4.is_some() || des_iface.ipv6.is_some() {
            iface_filter.include_ip_address = true;
        }
        let cur_iface = if let Ok(mut cur_ifaces) =
            get_ifaces_with_handle(&handle, Some(&iface_filter)).await
        {
            cur_ifaces.remove(&des_iface.name)
        } else {
            None
        };

        apply_iface_conf(&handle, des_iface, cur_iface.as_ref()).await?;
    }
    Ok(())
}
