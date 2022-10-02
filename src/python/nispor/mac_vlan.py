# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporMacVlan(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._mac_vlan_info = self._info.get("mac_vlan", {})

    @property
    def mode(self):
        return self._mac_vlan_info["mode"]

    @property
    def base_iface(self):
        return self._mac_vlan_info["base_iface"]

    @property
    def allowed_mac_addresses(self):
        return self._mac_vlan_info.get("allowed_mac_addresses")

    @property
    def mac_vlan_flags(self):
        return self._mac_vlan_info["flags"]
