// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, PartialEq, Eq, Clone)]
#[non_exhaustive]
/// The `NetStateIfaceFilter::default()` will retrieve full information.
/// To query only interested part, please use `NetStateIfaceFilter::minimum()`
/// along with additional property set to `Some()`.
pub struct NetStateIfaceFilter {
    /// Only specified interface. By default: None(all interfaces)
    pub iface_name: Option<String>,
    /// Include IP Address information. By default: true
    pub include_ip_address: bool,
    /// Include SR-IOV VF information or not. By default: true
    pub include_sriov_vf_info: bool,
    /// Include Bridge VLAN information or not. By default: true
    pub include_bridge_vlan: bool,
    /// Include ethool information or not. By default: true
    pub include_ethtool: bool,
    /// Include mptcp information or not. By default: true
    pub include_mptcp: bool,
}

impl Default for NetStateIfaceFilter {
    fn default() -> Self {
        Self {
            iface_name: None,
            include_ip_address: true,
            include_sriov_vf_info: true,
            include_bridge_vlan: true,
            include_ethtool: true,
            include_mptcp: true,
        }
    }
}

impl NetStateIfaceFilter {
    pub fn minimum() -> Self {
        Self {
            iface_name: None,
            include_ip_address: false,
            include_sriov_vf_info: false,
            include_bridge_vlan: false,
            include_ethtool: false,
            include_mptcp: false,
        }
    }
}
