// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};
use tokio::runtime;

use super::{
    conf::{apply_routes_conf, change_ifaces, create_ifaces, delete_ifaces},
    query::{get_iface_name2index, get_ifaces},
};
use crate::{IfaceConf, IfaceState, NisporError, RouteConf};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct NetConf {
    pub ifaces: Option<Vec<IfaceConf>>,
    pub routes: Option<Vec<RouteConf>>,
}

impl NetConf {
    pub fn apply(&self) -> Result<(), NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        rt.block_on(self.apply_async())
    }

    pub async fn apply_async(&self) -> Result<(), NisporError> {
        if let Some(ref ifaces) = &self.ifaces {
            let cur_iface_name_2_index = get_iface_name2index().await?;
            let mut new_ifaces = Vec::new();
            let mut del_ifaces = Vec::new();
            let mut chg_ifaces = Vec::new();
            for iface in ifaces {
                let cur_iface_index = cur_iface_name_2_index.get(&iface.name);
                if iface.state == IfaceState::Absent {
                    if let Some(cur_iface_index) = cur_iface_index {
                        del_ifaces
                            .push((iface.name.as_str(), *cur_iface_index));
                    }
                } else if cur_iface_index.is_none() {
                    new_ifaces.push(iface);
                    chg_ifaces.push(iface);
                } else {
                    chg_ifaces.push(iface);
                }
            }
            delete_ifaces(&del_ifaces).await?;
            create_ifaces(&new_ifaces, &cur_iface_name_2_index).await?;

            let cur_ifaces = get_ifaces(None).await?;
            change_ifaces(&chg_ifaces, &cur_ifaces).await?;
        }

        if let Some(routes) = self.routes.as_ref() {
            let cur_iface_name_2_index = get_iface_name2index().await?;
            apply_routes_conf(routes, &cur_iface_name_2_index).await?;
        }
        Ok(())
    }
}
