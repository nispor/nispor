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
from .base_iface import NisporBaseSubordinateIface


class NisporBond(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._bond_info = self._info.get("bond", {})

    @property
    def subordinates(self):
        return self._bond_info.get("subordinates")

    @property
    def mode(self):
        return self._bond_info.get("mode")

    @property
    def options(self):
        return self._bond_info.get("options")


class NisporBondSubordinate(NisporBaseSubordinateIface):
    def __init__(self, info):
        super().__init__()
        self._sub_info = info.get("bond_subordinate", {})

    @property
    def subordinate_state(self):
        return self._sub_info.get("subordinate_state")

    @property
    def mii_status(self):
        return self._sub_info.get("mii_status")

    @property
    def link_failure_count(self):
        return self._sub_info.get("link_failure_count")

    @property
    def perm_hwaddr(self):
        return self._sub_info.get("perm_hwaddr")

    @property
    def queue_id(self):
        return self._sub_info.get("queue_id")

    @property
    def ad_aggregator_id(self):
        return self._sub_info.get("ad_aggregator_id")

    @property
    def ad_actor_oper_port_state(self):
        return self._sub_info.get("ad_actor_oper_port_state")

    @property
    def ad_partner_oper_port_state(self):
        return self._sub_info.get("ad_partner_oper_port_state")
