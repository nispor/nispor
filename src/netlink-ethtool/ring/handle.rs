use crate::{EthtoolHandle, RingGetRequest};

pub struct RingHandle(EthtoolHandle);

impl RingHandle {
    pub fn new(handle: EthtoolHandle) -> Self {
        RingHandle(handle)
    }

    /// Retrieve the ethtool rings of a interface (equivalent to `ethtool -k eth1`)
    pub fn get(&mut self, iface_name: Option<&str>) -> RingGetRequest {
        RingGetRequest::new(self.0.clone(), iface_name)
    }
}
