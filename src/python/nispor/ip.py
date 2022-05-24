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


class NisporIPAddr:
    def __init__(self, info):
        self._info = info

    @property
    def address(self):
        return self._info["address"]

    @property
    def prefix_len(self):
        return self._info["prefix_len"]

    @property
    def valid_lft(self):
        return self._info["valid_lft"]

    @property
    def preferred_lft(self):
        return self._info["preferred_lft"]


class NisporIPv4Addr(NisporIPAddr):
    @property
    def peer(self):
        return self._info.get("peer")


class NisporIPv6Addr(NisporIPAddr):
    pass


class NisporIPv4:
    def __init__(self, info):
        self._info = info
        self._address = []
        for addr_info in info.get("addresses", []):
            self._address.append(NisporIPv4Addr(addr_info))

    @property
    def addresses(self):
        return self._address

    def __str__(self):
        return f"{self._info}"


class NisporIPv6:
    def __init__(self, info):
        self._info = info
        self._address = []
        for addr_info in info.get("addresses", []):
            self._address.append(NisporIPv6Addr(addr_info))

    @property
    def addresses(self):
        return self._address

    @property
    def addr_gen_mode(self):
        return self._info["addr_gen_mode"]

    def __str__(self):
        return f"{self._info}"
