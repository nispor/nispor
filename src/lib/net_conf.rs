use crate::error::NisporError;
use crate::ifaces::get_ifaces;
use crate::ifaces::IfaceConf;
use serde_derive::{Deserialize, Serialize};
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct NetConf {
    pub ifaces: Option<Vec<IfaceConf>>,
}

impl NetConf {
    // TODO: Return bool for whether change was made
    pub fn apply(&self) -> Result<(), NisporError> {
        let cur_ifaces = get_ifaces()?;
        if let Some(ifaces) = &self.ifaces {
            for iface in ifaces {
                if let Some(cur_iface) = cur_ifaces.get(&iface.name) {
                    Runtime::new()?.block_on(iface.apply(&cur_iface))?
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
