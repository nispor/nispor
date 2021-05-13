use crate::{CoalesceGetRequest, EthtoolHandle};

pub struct CoalesceHandle(EthtoolHandle);

impl CoalesceHandle {
    pub fn new(handle: EthtoolHandle) -> Self {
        CoalesceHandle(handle)
    }

    /// Retrieve the ethtool coalesces of a interface (equivalent to `ethtool -k eth1`)
    pub fn get(&mut self, iface_name: Option<&str>) -> CoalesceGetRequest {
        CoalesceGetRequest::new(self.0.clone(), iface_name)
    }
}
