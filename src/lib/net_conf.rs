// Copyright 2021 Red Hat, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::error::NisporError;
use crate::ifaces::{
    change_ifaces, create_ifaces, get_iface_name2index, get_ifaces, IfaceConf,
};

use serde::{Deserialize, Serialize};
use tokio::runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetConf {
    pub ifaces: Option<Vec<IfaceConf>>,
}

impl NetConf {
    pub fn apply(&self) -> Result<(), NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        let cur_iface_name_2_index = rt.block_on(get_iface_name2index())?;
        if let Some(ref ifaces) = &self.ifaces {
            let mut new_ifaces = Vec::new();
            for iface in ifaces {
                if let None = cur_iface_name_2_index.get(&iface.name) {
                    new_ifaces.push(iface);
                }
            }
            rt.block_on(create_ifaces(&new_ifaces))?;

            let cur_ifaces = rt.block_on(get_ifaces())?;
            rt.block_on(change_ifaces(ifaces.as_slice(), &cur_ifaces))?;
        }
        Ok(())
    }
}
