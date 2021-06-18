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
use crate::ifaces::get_ifaces;
use crate::ifaces::IfaceConf;
use serde::{Deserialize, Serialize};
use tokio::runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetConf {
    pub ifaces: Option<Vec<IfaceConf>>,
}

impl NetConf {
    // TODO: Return bool for whether change was made
    pub fn apply(&self) -> Result<(), NisporError> {
        let rt = runtime::Builder::new_current_thread().enable_io().build()?;
        let cur_ifaces = rt.block_on(get_ifaces())?;
        if let Some(ifaces) = &self.ifaces {
            for iface in ifaces {
                if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
                    rt.block_on(iface.apply(cur_iface))?
                } else {
                    // TODO: Create new interface
                    return Err(NisporError::invalid_argument(format!(
                        "Interface {} not found!",
                        iface.name
                    )));
                }
            }
        }
        Ok(())
    }
}
