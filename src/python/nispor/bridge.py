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


class NisporBridge(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._br_info = self._info.get("bridge")

    @property
    def ports(self):
        return self._br_info.get("ports")

    @property
    def subordinates(self):
        return self.ports

    @property
    def options(self):
        if self._br_info:
            return {
                key: value
                for key, value in self._br_info.items()
                if key != "ports"
            }
        return None


class NisporBridgePortVlan:
    def __init__(
        self, vid=None, vid_range=None, is_pvid=None, is_egress_untagged=None
    ):
        self.vid = vid
        self.vid_range = vid_range
        self.is_pvid = is_pvid
        self.is_egress_untagged = is_egress_untagged

    def __str__(self):
        return f"{self.__dict__}"


class NisporBridgePort(NisporBaseSubordinateIface):
    def __init__(self, info):
        super().__init__()
        self._sub_info = info.get("bridge_port")
        self._vlans = None
        if self._sub_info:
            self._vlans = []
            for vlan in self._sub_info.get("vlans", []):
                self._vlans.append(
                    NisporBridgePortVlan(
                        vlan.get("vid"),
                        vlan.get("vid_range"),
                        vlan.get("is_pvid"),
                        vlan.get("is_egress_untagged"),
                    )
                )

    @property
    def stp_state(self):
        return self._sub_info.get("stp_state")

    @property
    def stp_priority(self):
        return self._sub_info.get("stp_priority")

    @property
    def stp_path_cost(self):
        return self._sub_info.get("stp_path_cost")

    @property
    def hairpin_mode(self):
        return self._sub_info.get("hairpin_mode")

    @property
    def bpdu_guard(self):
        return self._sub_info.get("bpdu_guard")

    @property
    def root_block(self):
        return self._sub_info.get("root_block")

    @property
    def multicast_fast_leave(self):
        return self._sub_info.get("multicast_fast_leave")

    @property
    def learning(self):
        return self._sub_info.get("learning")

    @property
    def unicast_flood(self):
        return self._sub_info.get("unicast_flood")

    @property
    def proxyarp(self):
        return self._sub_info.get("proxyarp")

    @property
    def proxyarp_wifi(self):
        return self._sub_info.get("proxyarp_wifi")

    @property
    def designated_root(self):
        return self._sub_info.get("designated_root")

    @property
    def designated_bridge(self):
        return self._sub_info.get("designated_bridge")

    @property
    def designated_port(self):
        return self._sub_info.get("designated_port")

    @property
    def designated_cost(self):
        return self._sub_info.get("designated_cost")

    @property
    def port_id(self):
        return self._sub_info.get("port_id")

    @property
    def port_no(self):
        return self._sub_info.get("port_no")

    @property
    def change_ack(self):
        return self._sub_info.get("change_ack")

    @property
    def config_pending(self):
        return self._sub_info.get("config_pending")

    @property
    def message_age_timer(self):
        return self._sub_info.get("message_age_timer")

    @property
    def forward_delay_timer(self):
        return self._sub_info.get("forward_delay_timer")

    @property
    def hold_timer(self):
        return self._sub_info.get("hold_timer")

    @property
    def multicast_router(self):
        return self._sub_info.get("multicast_router")

    @property
    def multicast_flood(self):
        return self._sub_info.get("multicast_flood")

    @property
    def multicast_to_unicast(self):
        return self._sub_info.get("multicast_to_unicast")

    @property
    def vlan_tunnel(self):
        return self._sub_info.get("vlan_tunnel")

    @property
    def broadcast_flood(self):
        return self._sub_info.get("broadcast_flood")

    @property
    def group_fwd_mask(self):
        return self._sub_info.get("group_fwd_mask")

    @property
    def neigh_suppress(self):
        return self._sub_info.get("neigh_suppress")

    @property
    def isolated(self):
        return self._sub_info.get("isolated")

    @property
    def backup_port(self):
        return self._sub_info.get("backup_port")

    @property
    def vlans(self):
        return self._vlans
