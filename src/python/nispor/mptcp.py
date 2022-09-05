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


class NisporMptcpState:
    def __init__(self, info):
        self._addrs = [
            NisporMptcpAddress(addr_info) for addr_info in info.get("addresses", [])
        ]

    @property
    def enabled(self):
        return self._info["enabled"]

    @property
    def add_addr_accepted_limit(self):
        return self._info.get("add_addr_accepted_limit")

    @property
    def subflows_limit(self):
        return self._info.get("subflows_limit")

    @property
    def addresses(self):
        return self._addrs


class NisporMptcpAddress:
    def __init__(self, info):
        self._info = info

    @property
    def address(self):
        return self._info["address"]

    @property
    def id(self):
        return self._info.get("id")

    @property
    def port(self):
        return self._info.get("port")

    @property
    def flags(self):
        return self._info.get("flags")

    @property
    def iface(self):
        return self._info.get("iface")
