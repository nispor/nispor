use crate::{EthtoolHandle, LinkModeGetRequest};

pub struct LinkModeHandle(EthtoolHandle);

impl LinkModeHandle {
    pub fn new(handle: EthtoolHandle) -> Self {
        LinkModeHandle(handle)
    }

    /// Retrieve the ethtool link_modes of a interface (equivalent to `ethtool -k eth1`)
    pub fn get(&mut self, iface_name: Option<&str>) -> LinkModeGetRequest {
        LinkModeGetRequest::new(self.0.clone(), iface_name)
    }
}
