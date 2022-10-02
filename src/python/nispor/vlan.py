# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporVlan(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._vlan_info = self._info.get("vlan", {})

    @property
    def vlan_id(self):
        return self._vlan_info.get("vlan_id")

    @property
    def protocol(self):
        return self._vlan_info.get("protocol")

    @property
    def base_iface(self):
        return self._vlan_info.get("base_iface")

    @property
    def is_reorder_hdr(self):
        return self._vlan_info.get("is_reorder_hdr")

    @property
    def is_gvrp(self):
        return self._vlan_info.get("is_gvrp")

    @property
    def is_loose_binding(self):
        return self._vlan_info.get("is_loose_binding")

    @property
    def is_mvrp(self):
        return self._vlan_info.get("is_mvrp")

    @property
    def is_bridge_binding(self):
        return self._vlan_info.get("is_bridge_binding")
