// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct AltNameConf {
    #[serde(default)]
    remove: bool,
    name: String,
}

pub(crate) async fn change_alt_names(
    handle: &rtnetlink::Handle,
    ifaces: &[&IfaceConf],
    cur_ifaces: &HashMap<String, Iface>,
) -> Result<(), NisporError> {
    for iface in ifaces.iter().filter(|i| !i.alt_names.is_empty()) {
        if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
            let index = cur_iface.index;

            if is_all_remove(&iface.alt_names) {
                let names: Vec<&str> =
                    iface.alt_names.iter().map(|c| c.name.as_str()).collect();
                handle
                    .link()
                    .property_del(index)
                    .alt_ifname(&names)
                    .execute()
                    .await?;
            } else if is_all_add(&iface.alt_names) {
                let names: Vec<&str> =
                    iface.alt_names.iter().map(|c| c.name.as_str()).collect();
                handle
                    .link()
                    .property_add(index)
                    .alt_ifname(&names)
                    .execute()
                    .await?;
            } else {
                for alt_name_conf in iface.alt_names.iter() {
                    if alt_name_conf.remove {
                        handle
                            .link()
                            .property_del(index)
                            .alt_ifname(&[alt_name_conf.name.as_str()])
                            .execute()
                            .await?;
                    } else {
                        handle
                            .link()
                            .property_add(index)
                            .alt_ifname(&[alt_name_conf.name.as_str()])
                            .execute()
                            .await?;
                    }
                }
            }
        } else {
            return Err(NisporError::invalid_argument(format!(
                "Interface {} not found",
                iface.name
            )));
        }
    }
    Ok(())
}

fn is_all_remove(confs: &[AltNameConf]) -> bool {
    confs.iter().all(|c| c.remove)
}

fn is_all_add(confs: &[AltNameConf]) -> bool {
    confs.iter().all(|c| !c.remove)
}
