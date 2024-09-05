# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporIpVlan(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._ip_vlan_info = self._info.get("ip_vlan", {})

    @property
    def mode(self):
        return self._ip_vlan_info["mode"]

    @property
    def base_iface(self):
        return self._ip_vlan_info["base_iface"]

    @property
    def ip_vlan_flags(self):
        return self._ip_vlan_info["flags"]
