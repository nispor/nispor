# SPDX-License-Identifier: Apache-2.0

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
