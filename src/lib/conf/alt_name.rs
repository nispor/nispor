// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

use crate::{Iface, IfaceConf, NisporError};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
#[non_exhaustive]
pub struct AltNameConf {
    #[serde(default)]
    pub remove: bool,
    pub name: String,
}

impl AltNameConf {
    pub fn new(name: String, remove: bool) -> Self {
        Self { name, remove }
    }
}

pub(crate) async fn change_iface_alt_name(
    handle: &rtnetlink::Handle,
    iface: &IfaceConf,
    cur_iface: &Iface,
) -> Result<(), NisporError> {
    let iface_index = cur_iface.index;

    if is_all_remove(&iface.alt_names) {
        let names: Vec<&str> =
            iface.alt_names.iter().map(|c| c.name.as_str()).collect();
        handle
            .link()
            .property_del(iface_index)
            .alt_ifname(&names)
            .execute()
            .await?;
    } else if is_all_add(&iface.alt_names) {
        let names: Vec<&str> =
            iface.alt_names.iter().map(|c| c.name.as_str()).collect();
        handle
            .link()
            .property_add(iface_index)
            .alt_ifname(&names)
            .execute()
            .await?;
    } else {
        for alt_name_conf in iface.alt_names.iter() {
            if alt_name_conf.remove {
                handle
                    .link()
                    .property_del(iface_index)
                    .alt_ifname(&[alt_name_conf.name.as_str()])
                    .execute()
                    .await?;
            } else {
                handle
                    .link()
                    .property_add(iface_index)
                    .alt_ifname(&[alt_name_conf.name.as_str()])
                    .execute()
                    .await?;
            }
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
