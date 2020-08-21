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


class NisporSriov:
    def __init__(self, info):
        self._vfs = [NisporSriovVf(vf_info) for vf_info in info["vfs"]]

    @property
    def vfs(self):
        return self._vfs


class NisporSriovVf:
    def __init__(self, info):
        self._info = info

    @property
    def vf_id(self):
        return self._info["id"]

    @property
    def mac(self):
        return self._info["mac"]

    @property
    def broadcast(self):
        return self._info["broadcast"]

    @property
    def vlan_id(self):
        return self._info["vlan_id"]

    @property
    def qos(self):
        return self._info["qos"]

    @property
    def tx_rate(self):
        return self._info["tx_rate"]

    @property
    def spoof_check(self):
        return self._info["spoof_check"]

    @property
    def link_state(self):
        return self._info["link_state"]

    @property
    def min_tx_rate(self):
        return self._info["min_tx_rate"]

    @property
    def max_tx_rate(self):
        return self._info["max_tx_rate"]

    @property
    def query_rss(self):
        return self._info["query_rss"]

    @property
    def state(self):
        return NisporSriovVfState(self._info["state"])

    @property
    def trust(self):
        return self._info["trust"]

    @property
    def ib_node_guid(self):
        return self._info.get("ib_node_guid")

    @property
    def ib_port_guid(self):
        return self._info.get("ib_port_guid")


class NisporSriovVfState:
    def __init__(self, info):
        self._info = info

    @property
    def rx_packets(self):
        return self._info["rx_packets"]

    @property
    def tx_packets(self):
        return self._info["tx_packets"]

    @property
    def rx_bytes(self):
        return self._info["rx_bytes"]

    @property
    def tx_bytes(self):
        return self._info["tx_bytes"]

    @property
    def broadcast(self):
        return self._info["broadcast"]

    @property
    def multicast(self):
        return self._info["multicast"]

    @property
    def rx_dropped(self):
        return self._info["rx_dropped"]

    @property
    def tx_dropped(self):
        return self._info["tx_dropped"]
