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
        return self._bond_info["subordinates"]

    @property
    def mode(self):
        return self._bond_info["mode"]

    @property
    def miimon(self):
        return self._bond_info.get("miimon")

    @property
    def updelay(self):
        return self._bond_info.get("updelay")

    @property
    def downdelay(self):
        return self._bond_info.get("downdelay")

    @property
    def use_carrier(self):
        return self._bond_info.get("use_carrier")

    @property
    def arp_interval(self):
        return self._bond_info.get("arp_interval")

    @property
    def arp_ip_target(self):
        return self._bond_info.get("arp_ip_target")

    @property
    def arp_all_targets(self):
        return self._bond_info.get("arp_all_targets")

    @property
    def arp_validate(self):
        return self._bond_info.get("arp_validate")

    @property
    def primary(self):
        return self._bond_info.get("primary")

    @property
    def primary_reselect(self):
        return self._bond_info.get("primary_reselect")

    @property
    def fail_over_mac(self):
        return self._bond_info.get("fail_over_mac")

    @property
    def xmit_hash_policy(self):
        return self._bond_info.get("xmit_hash_policy")

    @property
    def resend_igmp(self):
        return self._bond_info.get("resend_igmp")

    @property
    def num_unsol_na(self):
        return self._bond_info.get("num_unsol_na")

    @property
    def num_grat_arp(self):
        return self._bond_info.get("num_grat_arp")

    @property
    def all_subordinates_active(self):
        return self._bond_info.get("all_subordinates_active")

    @property
    def min_links(self):
        return self._bond_info.get("min_links")

    @property
    def lp_interval(self):
        return self._bond_info.get("lp_interval")

    @property
    def packets_per_subordinate(self):
        return self._bond_info.get("packets_per_subordinate")

    @property
    def lacp_rate(self):
        return self._bond_info.get("lacp_rate")

    @property
    def ad_select(self):
        return self._bond_info.get("ad_select")

    @property
    def ad_actor_sys_prio(self):
        return self._bond_info.get("ad_actor_sys_prio")

    @property
    def ad_user_port_key(self):
        return self._bond_info.get("ad_user_port_key")

    @property
    def ad_actor_system(self):
        return self._bond_info.get("ad_actor_system")

    @property
    def tlb_dynamic_lb(self):
        return self._bond_info.get("tlb_dynamic_lb")

    @property
    def peer_notif_delay(self):
        return self._bond_info.get("peer_notif_delay")

    @property
    def ad_info(self):
        return self._bond_info.get("ad_info")


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
