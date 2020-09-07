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


class NisporTun(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._tun_info = self._info.get("tun", {})

    @property
    def mode(self):
        return self._tun_info["mode"]

    @property
    def owner(self):
        return self._tun_info.get("owner")

    @property
    def group(self):
        return self._tun_info.get("group")

    @property
    def pi(self):
        return self._tun_info["pi"]

    @property
    def vnet_hdr(self):
        return self._tun_info["vnet_hdr"]

    @property
    def multi_queue(self):
        return self._tun_info["multi_queue"]

    @property
    def persist(self):
        return self._tun_info["persist"]

    @property
    def num_queues(self):
        return self._tun_info.get("num_queues")

    @property
    def num_disabled_queues(self):
        return self._tun_info.get("num_disabled_queues")
