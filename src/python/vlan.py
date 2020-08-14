# Copyright 2020 Red Hat
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

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
