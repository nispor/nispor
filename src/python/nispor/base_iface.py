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

from .ip import NisporIPv4
from .ip import NisporIPv6
from .sr_iov import NisporSriov


class NisporBaseIface:
    def __init__(self, info):
        self._info = info
        self._sub_state = None
        self._sr_iov = None
        if "sriov" in self._info:
            self._sr_iov = NisporSriov(self._info["sriov"])

    def __str__(self):
        return f"{self._info}"

    @property
    def type(self):
        return self._info["iface_type"]

    @property
    def name(self):
        return self._info["name"]

    @property
    def state(self):
        return self._info["state"]

    @property
    def mtu(self):
        return self._info["mtu"]

    @property
    def flags(self):
        return self._info["flags"]

    @property
    def ipv4(self):
        if "ipv4" in self._info:
            return NisporIPv4(self._info["ipv4"])
        else:
            return None

    @property
    def ipv6(self):
        if "ipv6" in self._info:
            return NisporIPv6(self._info["ipv6"])
        else:
            return None

    @property
    def mac_address(self):
        return self._info.get("mac_address")

    @property
    def controller(self):
        return self._info.get("controller")

    @property
    def controller_type(self):
        return self._info.get("controller_type")

    @property
    def subordinate_state(self):
        return self._sub_state

    @subordinate_state.setter
    def subordinate_state(self, value):
        self._sub_state = value

    @property
    def sr_iov(self):
        return self._sr_iov


class NisporBaseSubordinateIface:
    def __init__(self):
        self._sub_info = None

    def __str__(self):
        return f"{self._sub_info}"
