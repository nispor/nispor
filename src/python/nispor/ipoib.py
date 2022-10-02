# SPDX-License-Identifier: Apache-2.0

from .base_iface import NisporBaseIface


class NisporIpoib(NisporBaseIface):
    def __init__(self, info):
        super().__init__(info)
        self._ipoib_info = self._info.get("ipoib", {})

    @property
    def pkey(self):
        return self._ipoib_info["pkey"]

    @property
    def mode(self):
        return self._ipoib_info["mode"]

    @property
    def umcast(self):
        return self._ipoib_info["umcast"]

    @property
    def base_iface(self):
        return self._ipoib_info.get("base_iface")
