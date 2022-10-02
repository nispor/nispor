# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporMacVtap(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._mac_vtap_info = self._info.get("mac_vtap", {})

    @property
    def mode(self):
        return self._mac_vtap_info["mode"]

    @property
    def base_iface(self):
        return self._mac_vtap_info["base_iface"]

    @property
    def allowed_mac_addresses(self):
        return self._mac_vtap_info.get("allowed_mac_addresses")

    @property
    def mac_vlan_flags(self):
        return self._mac_vtap_info["flags"]
