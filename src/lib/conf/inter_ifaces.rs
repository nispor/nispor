// SPDX-License-Identifier: Apache-2.0

use rtnetlink::new_connection;

use super::iface::apply_iface_conf;
use crate::{IfaceConf, NisporError};

pub(crate) async fn apply_ifaces_conf(
    des_ifaces: &[IfaceConf],
) -> Result<(), NisporError> {
    let (connection, handle, _) = new_connection()?;
    tokio::spawn(connection);
    for des_iface in des_ifaces {
        apply_iface_conf(&handle, des_iface).await?;
    }
    Ok(())
}
