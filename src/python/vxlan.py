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


class NisporVxlan(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._vxlan_info = self._info.get("vxlan", {})

    @property
    def vlan_id(self):
        return self._vlan_info.get("vlan_id")

    @property
    def remote(self):
        return self._vxlan_info.get("remote")

    @property
    def vxlan_id(self):
        return self._vxlan_info.get("vxlan_id")

    @property
    def base_iface(self):
        return self._vxlan_info.get("base_iface")

    @property
    def local(self):
        return self._vxlan_info.get("local")

    @property
    def ttl(self):
        return self._vxlan_info.get("ttl")

    @property
    def tos(self):
        return self._vxlan_info.get("tos")

    @property
    def learning(self):
        return self._vxlan_info.get("learning")

    @property
    def ageing(self):
        return self._vxlan_info.get("ageing")

    @property
    def max_address(self):
        return self._vxlan_info.get("max_address")

    @property
    def src_port_min(self):
        return self._vxlan_info.get("src_port_min")

    @property
    def src_port_max(self):
        return self._vxlan_info.get("src_port_max")

    @property
    def proxy(self):
        return self._vxlan_info.get("proxy")

    @property
    def rsc(self):
        return self._vxlan_info.get("rsc")

    @property
    def l2miss(self):
        return self._vxlan_info.get("l2miss")

    @property
    def l3miss(self):
        return self._vxlan_info.get("l3miss")

    @property
    def dst_port_min(self):
        return self._vxlan_info.get("dst_port_min")

    @property
    def dst_port_max(self):
        return self._vxlan_info.get("dst_port_max")

    @property
    def udp_check_sum(self):
        return self._vxlan_info.get("udp_check_sum")

    @property
    def udp6_zero_check_sum_tx(self):
        return self._vxlan_info.get("udp6_zero_check_sum_tx")

    @property
    def udp6_zero_check_sum_rx(self):
        return self._vxlan_info.get("udp6_zero_check_sum_rx")

    @property
    def remote_check_sum_tx(self):
        return self._vxlan_info.get("remote_check_sum_tx")

    @property
    def remote_check_sum_rx(self):
        return self._vxlan_info.get("remote_check_sum_rx")

    @property
    def gbp(self):
        return self._vxlan_info.get("gbp")

    @property
    def remote_check_sum_no_partial(self):
        return self._vxlan_info.get("remote_check_sum_no_partial")

    @property
    def collect_metadata(self):
        return self._vxlan_info.get("collect_metadata")

    @property
    def label(self):
        return self._vxlan_info.get("label")

    @property
    def gpe(self):
        return self._vxlan_info.get("gpe")

    @property
    def ttl_inherit(self):
        return self._vxlan_info.get("ttl_inherit")

    @property
    def df(self):
        return self._vxlan_info.get("df")
