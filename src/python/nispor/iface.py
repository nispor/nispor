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
from .bond import NisporBond
from .bond import NisporBondSubordinate
from .bridge import NisporBridge
from .bridge import NisporBridgePort
from .veth import NisporVeth
from .vlan import NisporVlan
from .vxlan import NisporVxlan


class NisporIfaceState:
    def __init__(self, info):
        self._ifaces = {}
        if info:
            for iface_name, iface_info in info.items():
                self._ifaces[iface_name] = _iface_info_to_obj(iface_info)

    def __str__(self):
        return f"{self._ifaces}"

    def get(self, iface_name):
        return self._ifaces.get(iface_name)

    def __getitem__(self, iface_name):
        return self._ifaces[iface_name]

    def keys(self):
        for iface in self._ifaces.keys():
            yield iface

    def values(self):
        for iface in self._ifaces.values():
            yield iface


def _iface_info_to_obj(iface_info):
    iface_type = iface_info["iface_type"]
    ctrl_type = iface_info.get("controller_type")
    if iface_type == "Bond":
        iface = NisporBond(iface_info)
    elif iface_type == "Bridge":
        iface = NisporBridge(iface_info)
    elif iface_type == "Vlan":
        iface = NisporVlan(iface_info)
    elif iface_type == "Vxlan":
        iface = NisporVxlan(iface_info)
    elif iface_type == "Veth":
        iface = NisporVeth(iface_info)
    else:
        iface = NisporBaseIface(iface_info)
    if ctrl_type == "Bond":
        iface.subordinate_state = NisporBondSubordinate(iface_info)
    elif ctrl_type == "Bridge":
        iface.subordinate_state = NisporBridgePort(iface_info)
    return iface
