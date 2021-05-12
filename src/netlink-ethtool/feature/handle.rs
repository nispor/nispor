use crate::{EthtoolHandle, FeatureGetRequest};

pub struct FeatureHandle(EthtoolHandle);

impl FeatureHandle {
    pub fn new(handle: EthtoolHandle) -> Self {
        FeatureHandle(handle)
    }

    /// Retrieve the ethtool features of a interface (equivalent to `ethtool -k eth1`)
    pub fn get(&mut self, iface_name: Option<&str>) -> FeatureGetRequest {
        FeatureGetRequest::new(self.0.clone(), iface_name)
    }
}
