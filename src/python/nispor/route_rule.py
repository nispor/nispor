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


class NisporRouteRuleState:
    def __init__(self, info):
        self._rls = [NisporRouteRule(rl_info) for rl_info in info]

    def __iter__(self):
        for rl in self._rls:
            yield rl


class NisporRouteRule:
    def __init__(self, info):
        self._info = info

    @property
    def action(self):
        return self._info["action"]

    @property
    def address_family(self):
        return self._info["address_family"]

    @property
    def flags(self):
        return self._info["flags"]

    @property
    def tos(self):
        return self._info["tos"]

    @property
    def table(self):
        return self._info.get("table")

    @property
    def dst(self):
        return self._info.get("dst")

    @property
    def src(self):
        return self._info.get("src")

    @property
    def iif(self):
        return self._info.get("iif")

    @property
    def oif(self):
        return self._info.get("oif")

    @property
    def goto(self):
        return self._info.get("goto")

    @property
    def priority(self):
        return self._info.get("priority")

    @property
    def fw_mark(self):
        return self._info.get("fw_mark")

    @property
    def fw_mask(self):
        return self._info.get("fw_mask")

    @property
    def mask(self):
        return self._info.get("mask")

    @property
    def flow(self):
        return self._info.get("flow")

    @property
    def tun_id(self):
        return self._info.get("tun_id")

    @property
    def suppress_ifgroup(self):
        return self._info.get("suppress_ifgroup")

    @property
    def suppress_prefix_len(self):
        return self._info.get("suppress_prefix_len")

    @property
    def protocol(self):
        return self._info.get("protocol")

    @property
    def ip_proto(self):
        return self._info.get("ip_proto")

    @property
    def src_port_range(self):
        return self._info.get("src_port_range")

    @property
    def dst_port_range(self):
        return self._info.get("dst_port_range")
