# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporVeth(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._peer = self._info.get("veth", {}).get("peer")

    @property
    def peer(self):
        return self._peer
